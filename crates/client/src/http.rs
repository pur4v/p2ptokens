//! The client's local HTTP surface — a drop-in chat-completions endpoint any
//! client or URL can point at (the `/v1/chat/completions` shape is the de-facto
//! standard used across the ecosystem):
//!   - `GET  /`                     the ASCII torrent-style dashboard
//!   - `GET  /api/status`           JSON status (peer id, ratio, offers, swarm)
//!   - `GET  /v1/models`            model list (network-wide)
//!   - `POST /v1/chat/completions`  completion routed to a peer (stream or not)

use std::convert::Infallible;
use std::sync::atomic::Ordering;

use axum::{
    extract::State,
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        Html, IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use p2ptokens_shared::types::{ChatCompletionParams, ChatMessage, ModelId};

use crate::ctx::{now_ms, SharedCtx};
use crate::leecher::{fan_out, leech, leech_stream, Fanout, StreamItem};

pub fn router(ctx: SharedCtx) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/config", get(config))
        .route("/api/status", get(status))
        .route("/v1/models", get(models))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(ctx)
}

async fn index() -> Html<&'static str> {
    Html(include_str!("ui.html"))
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

    // Fan-out modes coordinate across peers, so they are non-streaming.
    if mode != Fanout::Single {
        return match fan_out(&ctx, model, req.messages, params, mode, count).await {
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
        };
    }

    if req.stream.unwrap_or(false) {
        return stream_response(ctx, model, req.messages, params, id, created, model_name).await;
    }

    match leech(&ctx, model, req.messages, params).await {
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
    }
}

/// Server-sent-events streaming response of `chat.completion.chunk` objects.
async fn stream_response(
    ctx: SharedCtx,
    model: ModelId,
    messages: Vec<ChatMessage>,
    params: ChatCompletionParams,
    id: String,
    created: u64,
    model_name: String,
) -> Response {
    let mut rx = leech_stream(ctx, model, messages, params);

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
                StreamItem::Done { finish_reason, .. } => {
                    yield Ok(Event::default().data(chunk(json!({}), json!(finish_reason))));
                    yield Ok(Event::default().data("[DONE]".to_string()));
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
