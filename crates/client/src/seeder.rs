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

    ctx.in_flight.fetch_add(1, Ordering::SeqCst);
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

    // Settle the highest-seq consumer-signed receipt with the coordinator.
    if let Some(receipt) = latest {
        match ctx
            .coord
            .settle(&SettleRequest {
                receipt,
                consumer_pubkey,
                completed: true,
            })
            .await
        {
            Ok(resp) => tracing::info!(
                job = job_id,
                model = model_key,
                served_total = resp.provider_served_total,
                "job settled"
            ),
            Err(e) => tracing::warn!(job = job_id, "settle failed: {e:#}"),
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
