//! Seeder (provider) side: accept inbound completion streams, serve them from a
//! local backend adapter, and run the provider half of the co-receipt protocol
//! (design Q18). Settle the highest-seq consumer-signed receipt afterward.

use std::sync::atomic::Ordering;

use anyhow::Result;
use futures::StreamExt;
use libp2p::PeerId;
use tokio::sync::mpsc;

use p2ptokens_shared::api::SettleRequest;
use p2ptokens_shared::crypto;
use p2ptokens_shared::protocol::{read_msg, write_msg, Wire};
use p2ptokens_shared::receipts::{verify_receipt, SignedReceipt};

use crate::adapters::AdapterRequest;
use crate::ctx::SharedCtx;

/// Flush a chunk (and request a receipt) roughly every this many output tokens.
const CHUNK_TOKENS: u64 = 16;

pub async fn serve(ctx: SharedCtx, mut incoming: libp2p_stream::IncomingStreams) {
    while let Some((peer, stream)) = incoming.next().await {
        let ctx = ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = handle(ctx, peer, stream).await {
                tracing::warn!(%peer, "serve error: {e:#}");
            }
        });
    }
}

async fn handle(ctx: SharedCtx, peer: PeerId, mut s: libp2p::Stream) -> Result<()> {
    let first = read_msg(&mut s).await?;
    let Some(Wire::Request {
        job_id,
        model,
        consumer_pubkey,
        messages,
        params,
    }) = first
    else {
        let _ = write_msg(
            &mut s,
            &Wire::Error {
                message: "expected request".into(),
            },
        )
        .await;
        return Ok(());
    };

    // The remote peer must own the public key it presents.
    let pk = match crypto::import_pubkey(&consumer_pubkey) {
        Ok(pk) if pk.to_peer_id() == peer => pk,
        _ => {
            let _ = write_msg(
                &mut s,
                &Wire::Error {
                    message: "consumer pubkey mismatch".into(),
                },
            )
            .await;
            return Ok(());
        }
    };

    // Enforce this seeder's own input-size limit (abuse guard): refuse to process
    // an oversized/hostile payload rather than feeding it to the backend.
    if ctx.max_input_bytes > 0 {
        let sz = serde_json::to_vec(&messages).map(|v| v.len()).unwrap_or(0) as u64;
        if sz > ctx.max_input_bytes {
            let _ = write_msg(
                &mut s,
                &Wire::Error {
                    message: format!(
                        "request too large: {sz} bytes exceeds this peer's limit ({})",
                        ctx.max_input_bytes
                    ),
                },
            )
            .await;
            return Ok(());
        }
    }
    // Cap output tokens to this seeder's configured maximum (defends against a
    // consumer requesting an unbounded generation).
    let mut params = params;
    if ctx.max_output_tokens > 0 {
        params.max_tokens = Some(match params.max_tokens {
            Some(n) => n.min(ctx.max_output_tokens),
            None => ctx.max_output_tokens,
        });
    }

    let Some(serve) = ctx.model_index.get(&model) else {
        let _ = write_msg(
            &mut s,
            &Wire::Error {
                message: format!("model not served: {model}"),
            },
        )
        .await;
        return Ok(());
    };
    let adapter_idx = serve.adapter;
    let backend_model = serve.name.clone();

    // Dial-time capacity gate: atomically reserve a slot; if we've been raced past
    // the capacity we advertised (two consumers grabbed the last slot), back out and
    // reject so the leecher re-matches to another seeder instead of overloading us.
    let reserved = ctx.in_flight.fetch_add(1, Ordering::SeqCst);
    if ctx.capacity > 0 && reserved >= ctx.capacity {
        ctx.in_flight.fetch_sub(1, Ordering::SeqCst);
        let _ = write_msg(
            &mut s,
            &Wire::Error {
                message: "provider at capacity".into(),
            },
        )
        .await;
        return Ok(());
    }
    let result = run_job(
        &ctx,
        &mut s,
        &job_id,
        &model,
        &pk,
        adapter_idx,
        backend_model,
        messages,
        params,
        consumer_pubkey,
    )
    .await;
    ctx.in_flight.fetch_sub(1, Ordering::SeqCst);
    result
}

#[allow(clippy::too_many_arguments)]
async fn run_job(
    ctx: &SharedCtx,
    s: &mut libp2p::Stream,
    job_id: &str,
    model_key: &str,
    consumer_pk: &crypto::PubKey,
    adapter_idx: usize,
    backend_model: String,
    messages: Vec<p2ptokens_shared::types::ChatMessage>,
    params: p2ptokens_shared::types::ChatCompletionParams,
    consumer_pubkey: String,
) -> Result<()> {
    let started = std::time::Instant::now();
    // Drive the adapter in a background task, streaming deltas over a channel.
    let (tx, mut rx) = mpsc::channel(64);
    let ctx2 = ctx.clone();
    let producer = tokio::spawn(async move {
        ctx2.adapters[adapter_idx]
            .stream(
                AdapterRequest {
                    model: backend_model,
                    messages,
                    params,
                },
                tx,
            )
            .await
    });

    let mut seq: u64 = 0;
    let mut last_acked: u64 = 0;
    let mut cumulative: u64 = 0;
    let mut pending = String::new();
    let mut finish = "stop".to_string();
    let mut latest: Option<SignedReceipt> = None;

    while let Some(delta) = rx.recv().await {
        if !delta.text.is_empty() {
            pending.push_str(&delta.text);
            cumulative = delta.cumulative_tokens.max(cumulative);
        }
        if let Some(fr) = &delta.finish_reason {
            finish = fr.clone();
        }
        let should_flush = delta.done || cumulative.saturating_sub(last_acked) >= CHUNK_TOKENS;
        if should_flush && (!pending.is_empty() || delta.done) {
            seq += 1;
            write_msg(
                s,
                &Wire::Chunk {
                    seq,
                    text: std::mem::take(&mut pending),
                    cumulative_tokens: cumulative,
                },
            )
            .await?;
            match await_receipt(s, job_id, consumer_pk, cumulative).await? {
                Some(r) => {
                    latest = Some(r);
                    last_acked = cumulative;
                }
                None => break, // consumer gone or cheating
            }
        }
        if delta.done {
            write_msg(
                s,
                &Wire::Done {
                    finish_reason: finish.clone(),
                    cumulative_tokens: cumulative,
                },
            )
            .await?;
            break;
        }
    }

    let _ = producer.await;

    // Update the serving-throughput EMA (tokens/sec) the heartbeat reports, so the
    // coordinator can prefer faster peers. Ignore tiny/degenerate samples.
    let secs = started.elapsed().as_secs_f64();
    if secs >= 0.5 && cumulative > 0 {
        let sample = cumulative as f64 / secs;
        let prev = f64::from_bits(ctx.tps_ema.load(Ordering::Relaxed));
        let ema = if prev <= 0.0 {
            sample
        } else {
            0.7 * prev + 0.3 * sample
        };
        ctx.tps_ema.store(ema.to_bits(), Ordering::Relaxed);
    }

    // Settle the highest-seq consumer-signed receipt with the coordinator. Retry a
    // few times with backoff so a transient coordinator hiccup doesn't drop credit
    // for work already done.
    if let Some(receipt) = latest {
        let req = SettleRequest {
            receipt,
            consumer_pubkey,
            completed: true,
        };
        let mut settled = false;
        for attempt in 0..3u32 {
            match ctx.coord.settle(&req).await {
                Ok(resp) => {
                    tracing::info!(
                        job = job_id,
                        model = model_key,
                        served_total = resp.provider_served_total,
                        "job settled"
                    );
                    settled = true;
                    break;
                }
                Err(e) => {
                    tracing::warn!(job = job_id, attempt, "settle failed: {e:#}");
                    tokio::time::sleep(std::time::Duration::from_millis(
                        200 * (attempt as u64 + 1),
                    ))
                    .await;
                }
            }
        }
        if !settled {
            tracing::warn!(job = job_id, "settle gave up after retries");
        }
    }
    Ok(())
}

/// Read receipts until one is a valid consumer acknowledgement of at least
/// `target` cumulative tokens. Invalid signature => stop (potential cheat).
async fn await_receipt(
    s: &mut libp2p::Stream,
    job_id: &str,
    consumer_pk: &crypto::PubKey,
    target: u64,
) -> Result<Option<SignedReceipt>> {
    loop {
        match read_msg(s).await? {
            Some(Wire::Receipt { receipt }) => {
                if receipt.body.job_id != job_id
                    || !verify_receipt(&receipt.body, &receipt.consumer_sig, consumer_pk)
                {
                    return Ok(None);
                }
                if receipt.body.cumulative_tokens >= target {
                    return Ok(Some(receipt));
                }
                // stale receipt for an earlier chunk; keep waiting
            }
            Some(_) => continue,
            None => return Ok(None),
        }
    }
}
