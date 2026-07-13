//! p2ptokens coordinator — the hybrid tracker (design Q3).
//!
//! It brokers matches, tracks the barter ratio ledger, and settles co-receipts.
//! It never sees or carries inference content: bytes flow peer-to-peer. State is
//! sharded (DashMap) so requests run in parallel across all cores on the
//! multi-threaded Tokio runtime.

mod state;

use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Json, Path, Request, State as AxState},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use clap::Parser;

use p2ptokens_shared::api::{
    ErrorResponse, MatchManyRequest, MatchManyResponse, MatchResponse, SettleRequest,
    SettleResponse,
};
use p2ptokens_shared::config::PlatformConfig;
use p2ptokens_shared::crypto;
use p2ptokens_shared::receipts::verify_settlement;
use p2ptokens_shared::types::{Heartbeat, LedgerEntry, Match, MatchRequest};

use state::{now_ms, validate_heartbeat, Job, State};

type Shared = Arc<State>;

#[derive(Parser)]
#[command(name = "p2p-coordinator", about = "p2ptokens tracker")]
struct Args {
    /// path to a p2ptokens.toml config file (else ./p2ptokens.toml or defaults)
    #[arg(long)]
    config: Option<String>,
    /// address to listen on (overrides config `coordinator.listen`)
    #[arg(long)]
    listen: Option<String>,
    /// shared join secret for a PRIVATE network (overrides config `network.join_secret`)
    #[arg(long)]
    join_secret: Option<String>,
    /// accept unsigned heartbeats (INSECURE — local load testing only)
    #[arg(long, default_value_t = false)]
    allow_unsigned_heartbeats: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    let cfg = PlatformConfig::load(args.config.as_deref());
    let listen = args
        .listen
        .unwrap_or_else(|| cfg.coordinator.listen.clone());
    let join_secret = args.join_secret.or_else(|| cfg.join_secret());

    if args.allow_unsigned_heartbeats {
        tracing::warn!("accepting UNSIGNED heartbeats — insecure, use for load testing only");
    }
    if join_secret.is_some() {
        tracing::info!(network = %cfg.network.id, "PRIVATE network — join secret required on all requests");
    } else {
        tracing::info!(network = %cfg.network.id, "open network");
    }
    let shared: Shared = Arc::new(State::new(args.allow_unsigned_heartbeats, join_secret));

    // Background job sweep: drop abandoned jobs (e.g. the peers a fan-out race
    // drops, or a consumer that died mid-stream) so they can't accumulate.
    {
        let st = shared.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tick.tick().await;
                let swept = st.sweep_jobs(state::JOB_TTL_MS);
                if swept > 0 {
                    tracing::debug!(swept, "swept abandoned jobs");
                }
            }
        });
    }

    let app = Router::new()
        .route("/health", get(health))
        .route("/heartbeat", post(heartbeat))
        .route("/match", post(do_match))
        .route("/match_many", post(do_match_many))
        .route("/settle", post(settle))
        .route("/ledger/{peer}", get(get_ledger))
        .route("/providers", get(list_providers))
        // Private-network gate: require the join secret on every request but the
        // health probe (no-op on an open network).
        .layer(middleware::from_fn_with_state(
            shared.clone(),
            require_secret,
        ))
        // Cap every request body: these are all tiny metadata payloads, so a
        // 32 KiB ceiling stops oversized/hostile bodies from being buffered.
        .layer(DefaultBodyLimit::max(32 * 1024))
        .with_state(shared);

    let listener = tokio::net::TcpListener::bind(&listen).await?;
    tracing::info!(
        "coordinator listening on {} ({} worker threads)",
        listen,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    );
    axum::serve(listener, app).await?;
    Ok(())
}

/// Liveness probe for load balancers / uptime checks.
async fn health() -> &'static str {
    "ok"
}

/// Private-network gate: when a join secret is configured, every request except
/// `/health` must carry `Authorization: Bearer <join_secret>`. No-op otherwise.
async fn require_secret(AxState(st): AxState<Shared>, req: Request, next: Next) -> Response {
    if let Some(secret) = &st.join_secret {
        if req.uri().path() != "/health" {
            let ok = req
                .headers()
                .get(axum::http::header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|t| t == secret)
                .unwrap_or(false);
            if !ok {
                return (StatusCode::UNAUTHORIZED, "join secret required").into_response();
            }
        }
    }
    next.run(req).await
}

async fn heartbeat(AxState(st): AxState<Shared>, Json(hb): Json<Heartbeat>) -> StatusCode {
    if let Err(reason) = validate_heartbeat(&hb) {
        tracing::debug!(peer = %hb.peer_id, reason, "rejected heartbeat");
        return StatusCode::BAD_REQUEST;
    }
    // Verify the signature (cheap structural checks already passed). Skipped only
    // when explicitly running in insecure load-test mode.
    if !st.allow_unsigned && !p2ptokens_shared::auth::verify_heartbeat(&hb) {
        tracing::debug!(peer = %hb.peer_id, "rejected heartbeat: bad signature");
        return StatusCode::UNAUTHORIZED;
    }
    st.register(hb);
    StatusCode::OK
}

async fn do_match(
    AxState(st): AxState<Shared>,
    Json(req): Json<MatchRequest>,
) -> Json<MatchResponse> {
    st.touch_ledger(&req.consumer);

    if !st.consumer_allowed(&req.consumer) {
        return Json(MatchResponse::RatioExceeded);
    }

    match st.select_provider(&req.model, &req.consumer) {
        None => Json(MatchResponse::NoProvider),
        Some((provider, multiaddrs, audit, concrete)) => {
            let job_id = uuid::Uuid::new_v4().to_string();
            st.jobs.insert(
                job_id.clone(),
                Job {
                    consumer: req.consumer.clone(),
                    provider: provider.clone(),
                    model: concrete.clone(),
                    audit,
                    created: now_ms(),
                },
            );
            Json(MatchResponse::Matched(Match {
                job_id,
                provider,
                multiaddrs,
                model: concrete,
                audit,
            }))
        }
    }
}

/// Fan-out matchmaking: return up to `count` DISTINCT providers for the model,
/// each with its own job, so the client can race / quorum / ensemble across them.
async fn do_match_many(
    AxState(st): AxState<Shared>,
    Json(req): Json<MatchManyRequest>,
) -> Json<MatchManyResponse> {
    st.touch_ledger(&req.consumer);
    if !st.consumer_allowed(&req.consumer) {
        return Json(MatchManyResponse::RatioExceeded);
    }

    // Clamp the fan-out width so one request can't create an unbounded number of
    // jobs / dial an unbounded number of peers.
    let k = (req.count.max(1) as usize).min(state::MAX_FANOUT);
    let picked = st.select_providers(&req.model, &req.consumer, k);

    let matches = picked
        .into_iter()
        .map(|(provider, multiaddrs, audit, concrete)| {
            let job_id = uuid::Uuid::new_v4().to_string();
            st.jobs.insert(
                job_id.clone(),
                Job {
                    consumer: req.consumer.clone(),
                    provider: provider.clone(),
                    model: concrete.clone(),
                    audit,
                    created: now_ms(),
                },
            );
            Match {
                job_id,
                provider,
                multiaddrs,
                model: concrete,
                audit,
            }
        })
        .collect();

    Json(MatchManyResponse::Matched { matches })
}

async fn settle(
    AxState(st): AxState<Shared>,
    Json(req): Json<SettleRequest>,
) -> Result<Json<SettleResponse>, (StatusCode, Json<ErrorResponse>)> {
    let err = |code: StatusCode, msg: &str| {
        (
            code,
            Json(ErrorResponse {
                error: msg.to_string(),
            }),
        )
    };

    // Signature verification is CPU work — do it BEFORE touching shared state,
    // so crypto never serializes other requests.
    let consumer_pk = crypto::import_pubkey(&req.consumer_pubkey)
        .map_err(|_| err(StatusCode::BAD_REQUEST, "bad consumer_pubkey"))?;

    let job = st
        .jobs
        .remove(&req.receipt.body.job_id)
        .map(|(_, j)| j)
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "unknown or already-settled job"))?;

    if !verify_settlement(
        &req.receipt,
        &consumer_pk,
        &req.receipt.body.job_id,
        &job.consumer,
        &job.provider,
    ) {
        // put the job back so a valid retry can still settle it
        st.jobs.insert(req.receipt.body.job_id.clone(), job);
        return Err(err(StatusCode::BAD_REQUEST, "receipt failed verification"));
    }

    let tokens = req.receipt.body.cumulative_tokens;

    let provider_total = {
        let mut p = st
            .ledger
            .entry(job.provider.clone())
            .or_insert_with(|| LedgerEntry::new(job.provider.clone(), now_ms()));
        p.served += tokens;
        if job.audit {
            p.audits_total += 1;
            if req.completed {
                p.audits_passed += 1;
            }
            p.reputation = if p.audits_total == 0 {
                1.0
            } else {
                p.audits_passed as f64 / p.audits_total as f64
            };
        }
        p.served
    };

    let consumer_total = {
        let mut c = st
            .ledger
            .entry(job.consumer.clone())
            .or_insert_with(|| LedgerEntry::new(job.consumer.clone(), now_ms()));
        c.consumed += tokens;
        c.consumed
    };

    Ok(Json(SettleResponse {
        accepted: true,
        provider_served_total: provider_total,
        consumer_consumed_total: consumer_total,
    }))
}

async fn get_ledger(
    AxState(st): AxState<Shared>,
    Path(peer): Path<String>,
) -> Result<Json<LedgerEntry>, StatusCode> {
    st.ledger
        .get(&peer)
        .map(|e| Json(e.value().clone()))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn list_providers(AxState(st): AxState<Shared>) -> Json<Vec<Heartbeat>> {
    let now = now_ms();
    let live: Vec<Heartbeat> = st
        .providers
        .iter()
        .filter(|p| now.saturating_sub(p.last_seen) < state::HEARTBEAT_TTL_MS)
        .map(|p| p.hb.clone())
        .collect();
    Json(live)
}
