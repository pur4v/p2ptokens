//! The client's local HTTP surface: the embedded chat web UI plus the APIs it and
//! any OpenAI-compatible tool use.
//!   - `GET  /` + assets           the embedded chat dashboard (Vite bundle)
//!   - `GET  /api/status`          JSON status (peer id, ratio, offers, swarm)
//!   - `GET  /v1/models`           model list (network-wide)
//!   - `POST /v1/chat/completions` completion routed to a peer (stream or not)
//!   - `POST /streaming/chat/start` chat-UI streaming (x-token event protocol)
//!   - `GET/DELETE /threads…`      local SQLite chat history

use std::convert::Infallible;
use std::sync::atomic::Ordering;

use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{header, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use p2ptokens_shared::types::{
    ChatCompletionParams, ChatMessage, ContentPart, ImageUrl, MessageContent, ModelId,
};

use crate::ctx::{now_ms, SharedCtx};
use crate::leecher::{fan_out, fan_out_stream, leech, leech_stream, Fanout, StreamItem};

/// Stream-event token the chat UI splits on (see synapse STREAM_EVENTS contract).
const STREAM_TOKEN: &str = "x-token-9f8a7c2bfa4e49bd83c6aef78b29c1d3: ";

/// The built web UI (Vite bundle + shell), embedded at compile time. `build.rs`
/// produces `webui/dist` before this crate compiles.
#[derive(rust_embed::RustEmbed)]
#[folder = "webui/dist"]
struct WebAssets;

pub fn router(ctx: SharedCtx) -> Router {
    Router::new()
        .route("/api/config", get(config))
        .route("/api/status", get(status))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat_completions))
        // chat UI backend (synapse-style streaming + local history)
        .route("/streaming/chat/start", post(streaming_chat_start))
        .route("/threads", get(list_threads))
        .route("/threads/thread/{id}/messages", get(thread_messages))
        .route("/threads/{id}", delete(delete_thread_route))
        // embedded web UI (SPA): serve any bundled file by path; unknown paths
        // fall back to index.html so the single-page app can route.
        .route("/", get(static_or_index))
        .fallback(get(static_or_index))
        .with_state(ctx)
}

fn serve_embedded(path: &str) -> Response {
    match WebAssets::get(path) {
        Some(content) => (
            [(
                header::CONTENT_TYPE,
                content.metadata.mimetype().to_string(),
            )],
            content.data.into_owned(),
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

/// Serve the requested bundled asset (e.g. `/chat.min.js`), or `index.html` for
/// any path that isn't a file (SPA fallback). API routes are matched earlier.
async fn static_or_index(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() || WebAssets::get(path).is_none() {
        return serve_embedded("index.html");
    }
    serve_embedded(path)
}

/// White-label branding + network identity for the dashboard to render.
async fn config(State(ctx): State<SharedCtx>) -> Json<Value> {
    let b = &ctx.brand;
    Json(json!({
        "product_name": b.product_name,
        "tagline": b.tagline,
        "accent": b.accent,
        "amber": b.amber,
        "website": b.website,
        "github": b.github,
        "support_email": b.support_email,
        "logo_url": b.logo_url,
        "network": ctx.network_id,
    }))
}

fn parse_model(s: &str) -> ModelId {
    match s.split_once('@') {
        Some((name, quant)) => ModelId {
            name: name.to_string(),
            quant: Some(quant.to_string()),
        },
        None => ModelId {
            name: s.to_string(),
            quant: None,
        },
    }
}

async fn status(State(ctx): State<SharedCtx>) -> Json<Value> {
    let peer = ctx.local_peer.to_string();
    let ledger = ctx.coord.ledger(&peer).await.ok().flatten();
    let (served, consumed, reputation, ratio) = match &ledger {
        Some(e) => {
            let r = if e.consumed == 0 {
                f64::INFINITY
            } else {
                e.served as f64 / e.consumed as f64
            };
            (e.served, e.consumed, e.reputation, r)
        }
        None => (0, 0, 1.0, f64::INFINITY),
    };

    let providers = ctx.coord.providers().await.unwrap_or_default();
    let swarm: Vec<Value> = providers
        .iter()
        .map(|p| {
            json!({
                "peer_id": p.peer_id,
                "models": p.offers.iter().map(|o| o.model.key()).collect::<Vec<_>>(),
                "capacity": p.capacity,
                "in_flight": p.in_flight,
            })
        })
        .collect();

    Json(json!({
        "peer_id": peer,
        "listen_addrs": ctx.node.listen_addrs().iter().map(|a| a.to_string()).collect::<Vec<_>>(),
        "served": served,
        "consumed": consumed,
        "ratio": if ratio.is_finite() { json!(ratio) } else { json!("inf") },
        "reputation": reputation,
        "offers": ctx.offers.iter().map(|o| json!({"model": o.model.key(), "backend": o.backend})).collect::<Vec<_>>(),
        "capacity": ctx.capacity,
        "in_flight": ctx.in_flight.load(Ordering::SeqCst),
        "swarm": swarm,
        "swarm_size": providers.len(),
    }))
}

async fn models(State(ctx): State<SharedCtx>) -> Json<Value> {
    let providers = ctx.coord.providers().await.unwrap_or_default();
    let mut keys: Vec<String> = providers
        .iter()
        .flat_map(|p| p.offers.iter().map(|o| o.model.key()))
        .collect();
    keys.sort();
    keys.dedup();
    let data: Vec<Value> = keys
        .into_iter()
        .map(|k| json!({"id": k, "object": "model", "owned_by": "p2ptokens"}))
        .collect();
    Json(json!({ "object": "list", "data": data }))
}

#[derive(Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    stop: Option<Vec<String>>,
    /// fan-out mode: "single" | "racing" | "quorum" | "ensemble" (aliases ok).
    #[serde(default)]
    fanout: Option<String>,
    /// how many peers to fan out to (fan-out modes only).
    #[serde(default)]
    fanout_count: Option<u32>,
}

/// Resolve a fan-out mode from a model-name prefix shortcut (`race:llama3`,
/// `quorum:...`, `ensemble:...`) so even vanilla OpenAI clients can select one.
/// Returns the detected mode (if any) and the model name with the prefix removed.
fn split_model_prefix(model: &str) -> (Option<Fanout>, String) {
    for (pfx, mode) in [
        ("racing:", Fanout::Racing),
        ("race:", Fanout::Racing),
        ("quorum:", Fanout::Quorum),
        ("vote:", Fanout::Quorum),
        ("ensemble:", Fanout::Ensemble),
        ("moa:", Fanout::Ensemble),
        ("single:", Fanout::Single),
    ] {
        if let Some(rest) = model.strip_prefix(pfx) {
            return (Some(mode), rest.trim().to_string());
        }
    }
    (None, model.to_string())
}

async fn chat_completions(State(ctx): State<SharedCtx>, Json(req): Json<ChatRequest>) -> Response {
    // Resolve the fan-out mode: explicit `fanout` field wins, else a model-name
    // prefix shortcut (race:/quorum:/ensemble:), else single.
    let (prefix_mode, clean_model) = split_model_prefix(&req.model);
    let mode = req
        .fanout
        .as_deref()
        .and_then(Fanout::parse)
        .or(prefix_mode)
        .unwrap_or(Fanout::Single);
    let count = req.fanout_count.unwrap_or(3);

    let model = parse_model(&clean_model);
    let params = ChatCompletionParams {
        temperature: req.temperature,
        top_p: req.top_p,
        max_tokens: req.max_tokens,
        stop: req.stop,
    };
    let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
    let created = now_ms() / 1000;
    let model_name = clean_model;

    let streaming = req.stream.unwrap_or(false);

    // Streaming paths. Single always streams; racing streams the winning peer.
    // Quorum/ensemble aggregate across peers, so they cannot stream and fall
    // through to the buffered fan-out below.
    if streaming && mode == Fanout::Single {
        let rx = leech_stream(ctx, model, req.messages, params);
        return stream_response(rx, id, created, model_name).await;
    }
    if streaming && mode == Fanout::Racing {
        let rx = fan_out_stream(ctx, model, req.messages, params, count);
        return stream_response(rx, id, created, model_name).await;
    }

    // Non-streaming single = one buffered completion.
    if mode == Fanout::Single {
        return match leech(&ctx, model, req.messages, params).await {
            Ok(res) => Json(json!({
                "id": id,
                "object": "chat.completion",
                "created": created,
                "model": model_name,
                "p2p_provider": res.provider,
                "choices": [{
                    "index": 0,
                    "message": { "role": "assistant", "content": res.text },
                    "finish_reason": res.finish_reason,
                }],
                "usage": {
                    "prompt_tokens": 0,
                    "completion_tokens": res.cumulative_tokens,
                    "total_tokens": res.cumulative_tokens,
                }
            }))
            .into_response(),
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": {"message": e.to_string(), "type": "p2p_error"}})),
            )
                .into_response(),
        };
    }

    // Buffered fan-out: racing without streaming, plus quorum / ensemble.
    match fan_out(&ctx, model, req.messages, params, mode, count).await {
        Ok(out) => {
            let choices: Vec<Value> = out
                .choices
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    json!({
                        "index": i,
                        "message": { "role": "assistant", "content": c.text },
                        "finish_reason": c.finish_reason,
                        "p2p_provider": c.provider,
                    })
                })
                .collect();
            let total: u64 = out.choices.iter().map(|c| c.cumulative_tokens).sum();
            Json(json!({
                "id": id,
                "object": "chat.completion",
                "created": created,
                "model": model_name,
                "choices": choices,
                "usage": {
                    "prompt_tokens": 0,
                    "completion_tokens": total,
                    "total_tokens": total,
                },
                "p2p_fanout": {
                    "mode": out.mode,
                    "responded": out.responded,
                    "agreement": out.agreement,
                },
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": {"message": e.to_string(), "type": "p2p_error"}})),
        )
            .into_response(),
    }
}

/// Server-sent-events streaming response of `chat.completion.chunk` objects,
/// driven by a [`StreamItem`] channel (single leech or racing fan-out).
async fn stream_response(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<StreamItem>,
    id: String,
    created: u64,
    model_name: String,
) -> Response {
    let chunk = move |delta: Value, finish: Value| -> String {
        json!({
            "id": id,
            "object": "chat.completion.chunk",
            "created": created,
            "model": model_name,
            "choices": [{ "index": 0, "delta": delta, "finish_reason": finish }],
        })
        .to_string()
    };

    let s = async_stream::stream! {
        while let Some(item) = rx.recv().await {
            match item {
                StreamItem::Delta(t) => {
                    yield Ok::<Event, Infallible>(
                        Event::default().data(chunk(json!({ "content": t }), Value::Null)),
                    );
                }
                StreamItem::Done { finish_reason } => {
                    yield Ok(Event::default().data(chunk(json!({}), json!(finish_reason))));
                    yield Ok(Event::default().data("[DONE]"));
                    break;
                }
                StreamItem::Err(e) => {
                    yield Ok(Event::default().data(
                        json!({"error": {"message": e, "type": "p2p_error"}}).to_string(),
                    ));
                    break;
                }
            }
        }
    };

    Sse::new(s).into_response()
}

// ==================== chat-UI backend (streaming + local history) ====================

/// Encode one stream event as a `x-token-…: {json}` frame (the chat UI splits on
/// the token prefix and JSON-parses each frame).
fn ev(kind: &str, mut extra: Value) -> Result<Bytes, Infallible> {
    if let Value::Object(m) = &mut extra {
        m.insert("type".into(), json!(kind));
    }
    Ok(Bytes::from(format!("{STREAM_TOKEN}{extra}")))
}

#[derive(Deserialize, Default)]
struct StreamMsg {
    #[serde(default)]
    content: String,
}

/// One attachment as the UI sends it (`UploadedImage`: `{type, filename, url}`).
#[derive(Deserialize, Default)]
struct Attach {
    #[serde(default)]
    url: String,
    #[serde(default)]
    filename: String,
    #[serde(default, rename = "type")]
    mime: String,
}

#[derive(Deserialize)]
struct StreamStartReq {
    #[serde(default)]
    message: StreamMsg,
    #[serde(default)]
    thread_id: Option<String>,
    #[serde(default)]
    model: Option<String>,
    /// fan-out strategy: single | racing | quorum | ensemble
    #[serde(default)]
    strategy: Option<String>,
    #[serde(default)]
    count: Option<u32>,
    #[serde(default)]
    images: Vec<Attach>,
}

fn title_from(text: &str) -> String {
    let t = text.trim();
    if t.is_empty() {
        "New chat".to_string()
    } else {
        t.chars().take(48).collect()
    }
}

/// Build multimodal message content from the prompt text + attachments.
fn build_user_content(text: &str, atts: &[Attach]) -> MessageContent {
    if atts.is_empty() {
        return MessageContent::Text(text.to_string());
    }
    let mut parts = Vec::new();
    if !text.is_empty() {
        parts.push(ContentPart::Text {
            text: text.to_string(),
        });
    }
    for a in atts {
        if a.mime.starts_with("image/") || a.url.starts_with("data:image/") {
            parts.push(ContentPart::ImageUrl {
                image_url: ImageUrl { url: a.url.clone() },
            });
        } else {
            parts.push(ContentPart::File {
                file: p2ptokens_shared::types::FileData {
                    filename: a.filename.clone(),
                    mime: a.mime.clone(),
                    data: a.url.clone(),
                },
            });
        }
    }
    MessageContent::Parts(parts)
}

/// Streaming chat endpoint that drives the embedded chat UI. Emits the
/// `x-token-…` event protocol (thread_created → user_message_created → streaming
/// deltas → ai_message_created → stream_end) over the P2P inference path, and
/// persists the turn to the local SQLite history.
async fn streaming_chat_start(
    State(ctx): State<SharedCtx>,
    Json(req): Json<StreamStartReq>,
) -> Response {
    let (prefix_mode, clean_model) = split_model_prefix(req.model.as_deref().unwrap_or_default());
    let mode = req
        .strategy
        .as_deref()
        .and_then(Fanout::parse)
        .or(prefix_mode)
        .unwrap_or(Fanout::Single);
    let count = req.count.unwrap_or(3);
    let model = parse_model(&clean_model);

    let is_new = req.thread_id.as_deref().map(str::is_empty).unwrap_or(true);
    let thread_id = req
        .thread_id
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let now = now_ms() as i64;
    let title = title_from(&req.message.content);
    if is_new {
        let _ = ctx.threads.create_thread(&thread_id, &title, now);
    }

    // Persist the user turn (text + attachments) before streaming the answer.
    let user_content = build_user_content(&req.message.content, &req.images);
    let user_msg_id = uuid::Uuid::new_v4().to_string();
    let _ = ctx.threads.add_message(
        &user_msg_id,
        &thread_id,
        "user",
        &serde_json::to_string(&user_content).unwrap_or_default(),
        now,
    );

    // Context = the whole thread (includes the user turn we just saved).
    let messages = ctx.threads.history_as_chat(&thread_id).unwrap_or_default();
    let params = ChatCompletionParams::default();

    let body = async_stream::stream! {
        if is_new {
            yield ev("thread_created", json!({ "content": thread_id }));
            yield ev("thread_title", json!({ "content": title }));
        }
        yield ev("user_message_created", json!({ "content": user_msg_id }));

        let mut answer = String::new();
        match mode {
            Fanout::Single | Fanout::Racing => {
                let mut rx = if mode == Fanout::Racing {
                    fan_out_stream(ctx.clone(), model.clone(), messages.clone(), params.clone(), count)
                } else {
                    leech_stream(ctx.clone(), model.clone(), messages.clone(), params.clone())
                };
                while let Some(item) = rx.recv().await {
                    match item {
                        StreamItem::Delta(t) => {
                            answer.push_str(&t);
                            yield ev("streaming", json!({ "name": "p2ptokens", "content": t, "message": {}, "detail": {} }));
                        }
                        StreamItem::Done { .. } => break,
                        StreamItem::Err(e) => {
                            yield ev("error", json!({ "name": "STREAMING_ERROR", "content": e }));
                            break;
                        }
                    }
                }
            }
            _ => {
                match fan_out(&ctx, model.clone(), messages.clone(), params.clone(), mode, count).await {
                    Ok(out) => {
                        let text = out.choices.first().map(|c| c.text.clone()).unwrap_or_default();
                        answer = text.clone();
                        yield ev("streaming", json!({ "name": "p2ptokens", "content": text, "message": {}, "detail": {} }));
                    }
                    Err(e) => {
                        yield ev("error", json!({ "name": "STREAMING_ERROR", "content": e.to_string() }));
                    }
                }
            }
        }

        let ai_id = uuid::Uuid::new_v4().to_string();
        let _ = ctx.threads.add_message(
            &ai_id,
            &thread_id,
            "assistant",
            &serde_json::to_string(&MessageContent::Text(answer)).unwrap_or_default(),
            now_ms() as i64,
        );
        yield ev("ai_message_created", json!({ "content": ai_id }));
        yield ev("stream_end", json!({ "content": "" }));
    };

    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        Body::from_stream(body),
    )
        .into_response()
}

#[derive(Deserialize)]
struct ThreadsQuery {
    #[serde(default)]
    query: Option<String>,
}

async fn list_threads(
    State(ctx): State<SharedCtx>,
    axum::extract::Query(q): axum::extract::Query<ThreadsQuery>,
) -> Json<Value> {
    let rows = ctx
        .threads
        .list_threads(q.query.as_deref().filter(|s| !s.is_empty()))
        .unwrap_or_default();
    let threads: Vec<Value> = rows
        .iter()
        .map(
            |t| json!({ "id": t.id, "title": t.title, "created": t.created, "updated": t.updated }),
        )
        .collect();
    let total = threads.len();
    Json(json!({
        "threads": threads,
        "pagination": { "has_next": false, "page": 1, "total": total },
    }))
}

async fn thread_messages(State(ctx): State<SharedCtx>, Path(id): Path<String>) -> Json<Value> {
    let rows = ctx.threads.get_messages(&id).unwrap_or_default();
    let msgs: Vec<Value> = rows
        .iter()
        .map(|m| {
            let mc: MessageContent = serde_json::from_value(m.content.clone())
                .unwrap_or(MessageContent::Text(String::new()));
            let text = mc.to_text();
            if m.role == "assistant" {
                json!({
                    "role": "assistant",
                    "id": m.id,
                    "assistantChunks": [{ "id": m.id, "type": "streaming", "content": text, "detail": { "toolUsed": [] } }],
                })
            } else {
                json!({ "role": "user", "id": m.id, "content": text })
            }
        })
        .collect();
    Json(json!({ "success": true, "data": { "messages": msgs, "active_task": false } }))
}

async fn delete_thread_route(State(ctx): State<SharedCtx>, Path(id): Path<String>) -> StatusCode {
    let _ = ctx.threads.delete_thread(&id);
    StatusCode::OK
}
