//! Leecher (consumer) side: get matched by the coordinator, dial the seeder over
//! libp2p, stream the completion, and co-sign a receipt per chunk (design Q18).

use anyhow::{bail, Result};
use libp2p::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId, StreamProtocol};
use tokio::sync::mpsc;

use p2ptokens_shared::api::{MatchManyRequest, MatchResponse};
use p2ptokens_shared::protocol::{completion_protocol, read_msg, write_msg, Wire};
use p2ptokens_shared::receipts::{sign_receipt, ReceiptBody, SignedReceipt};
use p2ptokens_shared::types::{ChatCompletionParams, ChatMessage, Match, MatchRequest, ModelId};

use crate::ctx::{now_ms, SharedCtx};

pub struct CompletionResult {
    pub text: String,
    pub cumulative_tokens: u64,
    pub finish_reason: String,
    pub provider: String,
}

/// Streaming events emitted to a caller (used by the SSE endpoint).
pub enum StreamItem {
    Delta(String),
    Done {
        finish_reason: String,
        cumulative_tokens: u64,
        provider: String,
    },
    Err(String),
}

/// Aggregate a full completion (non-streaming callers).
pub async fn leech(
    ctx: &SharedCtx,
    model: ModelId,
    messages: Vec<ChatMessage>,
    params: ChatCompletionParams,
) -> Result<CompletionResult> {
    run(ctx, model, messages, params, |_| {}).await
}

/// Stream a completion, delivering deltas over a channel as they arrive.
pub fn leech_stream(
    ctx: SharedCtx,
    model: ModelId,
    messages: Vec<ChatMessage>,
    params: ChatCompletionParams,
) -> mpsc::UnboundedReceiver<StreamItem> {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let tx2 = tx.clone();
        let res = run(&ctx, model, messages, params, move |t| {
            let _ = tx2.send(StreamItem::Delta(t.to_string()));
        })
        .await;
        match res {
            Ok(r) => {
                let _ = tx.send(StreamItem::Done {
                    finish_reason: r.finish_reason,
                    cumulative_tokens: r.cumulative_tokens,
                    provider: r.provider,
                });
            }
            Err(e) => {
                let _ = tx.send(StreamItem::Err(e.to_string()));
            }
        }
    });
    rx
}

/// Core routine: fetch one match, then dial + stream + co-sign receipts.
async fn run<F: FnMut(&str)>(
    ctx: &SharedCtx,
    model: ModelId,
    messages: Vec<ChatMessage>,
    params: ChatCompletionParams,
    on_delta: F,
) -> Result<CompletionResult> {
    let match_resp = ctx
        .coord
        .request_match(&MatchRequest {
            consumer: ctx.local_peer.to_string(),
            model,
        })
        .await?;

    // Failure variants carry only a code on the wire; render the message here.
    let m = match match_resp {
        MatchResponse::Matched(m) => m,
        MatchResponse::NoProvider => bail!("no provider available for this model"),
        MatchResponse::RatioExceeded => {
            bail!("rate limited: serve more to restore your ratio before leeching")
        }
    };

    run_with_match(ctx, m, messages, params, on_delta).await
}

/// Dial a specific, pre-resolved match, stream the completion, and co-sign a
/// receipt per chunk. Shared by single leeching and by fan-out.
async fn run_with_match<F: FnMut(&str)>(
    ctx: &SharedCtx,
    m: Match,
    messages: Vec<ChatMessage>,
    params: ChatCompletionParams,
    mut on_delta: F,
) -> Result<CompletionResult> {
    // the coordinator resolved a concrete model (with quant) the provider serves
    let served_key = m.model.key();
    let provider: PeerId = m.provider.parse()?;
    // Addresses arrive BARE (coordinator stripped the trailing `/p2p/<provider>`
    // to save bytes); re-attach the provider id so circuit routing has its
    // destination. Identity is still enforced independently by
    // `DialOpts::peer_id(provider)` in the node, so this reconstruction cannot be
    // abused to point us at a different peer.
    let addrs: Vec<Multiaddr> = m
        .multiaddrs
        .iter()
        .filter_map(|a| a.parse::<Multiaddr>().ok())
        .map(|a| ensure_peer(a, provider))
        .collect();

    ctx.node.connect(provider, addrs).await?;
    let mut control = ctx.node.control();
    let proto = StreamProtocol::try_from_owned(completion_protocol(&ctx.network_id))?;
    let mut s = control.open_stream(provider, proto).await?;

    write_msg(
        &mut s,
        &Wire::Request {
            job_id: m.job_id.clone(),
            model: served_key.clone(),
            consumer_pubkey: ctx.pubkey_b64.clone(),
            messages,
            params,
        },
    )
    .await?;

    let mut text = String::new();
    let mut cumulative = 0u64;
    let mut finish = "stop".to_string();

    loop {
        match read_msg(&mut s).await? {
            Some(Wire::Chunk {
                seq,
                text: t,
                cumulative_tokens,
            }) => {
                on_delta(&t);
                text.push_str(&t);
                cumulative = cumulative_tokens;
                // co-sign acknowledgement of cumulative tokens received
                let body = ReceiptBody {
                    job_id: m.job_id.clone(),
                    consumer: ctx.local_peer.to_string(),
                    provider: m.provider.clone(),
                    model: served_key.clone(),
                    seq,
                    cumulative_tokens,
                    ts: now_ms(),
                };
                let sig = sign_receipt(&ctx.keypair, &body)?;
                write_msg(
                    &mut s,
                    &Wire::Receipt {
                        receipt: SignedReceipt {
                            body,
                            consumer_sig: sig,
                            provider_sig: None,
                        },
                    },
                )
                .await?;
            }
            Some(Wire::Done {
                finish_reason,
                cumulative_tokens,
            }) => {
                cumulative = cumulative_tokens.max(cumulative);
                finish = finish_reason;
                break;
            }
            Some(Wire::Error { message }) => bail!("provider error: {message}"),
            Some(_) => continue,
            None => break,
        }
    }

    Ok(CompletionResult {
        text,
        cumulative_tokens: cumulative,
        finish_reason: finish,
        provider: m.provider,
    })
}

/// Leech from a pre-resolved match (no streaming callback). Used by fan-out.
async fn leech_with_match(
    ctx: &SharedCtx,
    m: Match,
    messages: Vec<ChatMessage>,
    params: ChatCompletionParams,
) -> Result<CompletionResult> {
    run_with_match(ctx, m, messages, params, |_| {}).await
}

/// Fan-out strategy for a single prompt across multiple providers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fanout {
    /// one provider (default) — the only mode that streams.
    Single,
    /// dial K, return the first full completion, drop the rest.
    Racing,
    /// dial K, return the majority answer + how many agreed.
    Quorum,
    /// dial K, return every answer as a separate choice.
    Ensemble,
}

impl Fanout {
    /// Parse a mode from a user-supplied string (case-insensitive, with aliases).
    pub fn parse(s: &str) -> Option<Fanout> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "single" | "off" => Some(Fanout::Single),
            "racing" | "race" | "first" | "fastest" => Some(Fanout::Racing),
            "quorum" | "vote" | "redundant" => Some(Fanout::Quorum),
            "ensemble" | "moa" | "all" => Some(Fanout::Ensemble),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Fanout::Single => "single",
            Fanout::Racing => "racing",
            Fanout::Quorum => "quorum",
            Fanout::Ensemble => "ensemble",
        }
    }
}

/// Outcome of a fan-out: one or more candidate completions plus metadata.
pub struct FanoutOutcome {
    /// 1 for single/racing/quorum; up to K for ensemble.
    pub choices: Vec<CompletionResult>,
    pub mode: &'static str,
    /// quorum only: how many providers agreed on the returned answer.
    pub agreement: Option<usize>,
    /// how many providers returned a successful completion.
    pub responded: usize,
}

/// Fan one prompt out to up to `count` distinct providers per `mode`.
pub async fn fan_out(
    ctx: &SharedCtx,
    model: ModelId,
    messages: Vec<ChatMessage>,
    params: ChatCompletionParams,
    mode: Fanout,
    count: u32,
) -> Result<FanoutOutcome> {
    if mode == Fanout::Single || count <= 1 {
        let r = leech(ctx, model, messages, params).await?;
        return Ok(FanoutOutcome {
            choices: vec![r],
            mode: "single",
            agreement: None,
            responded: 1,
        });
    }

    // Ask the coordinator for up to `count` DISTINCT providers (each gets a job).
    let matches = ctx
        .coord
        .request_matches(&MatchManyRequest {
            consumer: ctx.local_peer.to_string(),
            model,
            count,
        })
        .await?;
    if matches.is_empty() {
        bail!("no providers available for fan-out");
    }

    use futures::stream::{FuturesUnordered, StreamExt};
    let mut tasks: FuturesUnordered<_> = matches
        .into_iter()
        .map(|m| leech_with_match(ctx, m, messages.clone(), params.clone()))
        .collect();

    match mode {
        Fanout::Racing => {
            // First successful completion wins; dropping `tasks` cancels the rest
            // (their libp2p streams close; the coordinator sweeps their jobs).
            while let Some(res) = tasks.next().await {
                if let Ok(r) = res {
                    return Ok(FanoutOutcome {
                        choices: vec![r],
                        mode: "racing",
                        agreement: None,
                        responded: 1,
                    });
                }
            }
            bail!("all providers failed")
        }
        _ => {
            // quorum / ensemble: gather every successful completion.
            let mut results = Vec::new();
            while let Some(res) = tasks.next().await {
                if let Ok(r) = res {
                    results.push(r);
                }
            }
            if results.is_empty() {
                bail!("all providers failed");
            }
            let responded = results.len();
            if mode == Fanout::Ensemble {
                Ok(FanoutOutcome {
                    choices: results,
                    mode: "ensemble",
                    agreement: None,
                    responded,
                })
            } else {
                let (winner, agree) = majority(results);
                Ok(FanoutOutcome {
                    choices: vec![winner],
                    mode: "quorum",
                    agreement: Some(agree),
                    responded,
                })
            }
        }
    }
}

/// Return the completion whose text is most common (ties → first), plus the
/// number of responses that agreed with it.
fn majority(mut results: Vec<CompletionResult>) -> (CompletionResult, usize) {
    let mut best_idx = 0;
    let mut best_count = 0;
    for i in 0..results.len() {
        let c = results.iter().filter(|r| r.text == results[i].text).count();
        if c > best_count {
            best_count = c;
            best_idx = i;
        }
    }
    let winner = results.swap_remove(best_idx);
    (winner, best_count)
}

/// Re-attach a provider's `PeerId` to a bare dial address. The coordinator ships
/// addresses without the trailing `/p2p/<provider>` to save bytes; this puts it
/// back so the address is canonical (and circuit-routable). Idempotent: if the
/// address already ends with this peer id, it is returned unchanged, so we never
/// double-append or clobber an intermediate relay id.
fn ensure_peer(addr: Multiaddr, peer: PeerId) -> Multiaddr {
    if matches!(addr.iter().last(), Some(Protocol::P2p(id)) if id == peer) {
        addr
    } else {
        addr.with(Protocol::P2p(peer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cr(text: &str, provider: &str) -> CompletionResult {
        CompletionResult {
            text: text.into(),
            cumulative_tokens: 0,
            finish_reason: "stop".into(),
            provider: provider.into(),
        }
    }

    #[test]
    fn majority_picks_the_most_common_answer() {
        // two peers say "42", one says "41" -> "42" wins with agreement 2
        let (winner, agree) = majority(vec![cr("41", "a"), cr("42", "b"), cr("42", "c")]);
        assert_eq!(winner.text, "42");
        assert_eq!(agree, 2);
    }

    #[test]
    fn majority_all_disagree_returns_first() {
        let (winner, agree) = majority(vec![cr("x", "a"), cr("y", "b")]);
        assert_eq!(winner.text, "x");
        assert_eq!(agree, 1);
    }

    // A bare address + provider id must round-trip back to the canonical
    // dialable form, for both direct and relayed addresses, and be idempotent.
    #[test]
    fn ensure_peer_roundtrips_and_is_idempotent() {
        let provider = PeerId::random();
        let relay = PeerId::random();

        let direct: Multiaddr = "/ip4/203.0.113.7/tcp/40833".parse().unwrap();
        let relayed: Multiaddr = format!("/ip4/198.51.100.9/tcp/4001/p2p/{relay}/p2p-circuit")
            .parse()
            .unwrap();

        for bare in [direct, relayed] {
            let once = ensure_peer(bare.clone(), provider);
            // trailing component is the provider id
            assert!(matches!(once.iter().last(), Some(Protocol::P2p(id)) if id == provider));
            // idempotent — re-attaching does not duplicate
            assert_eq!(once, ensure_peer(once.clone(), provider));
            // the relay id (if any) is preserved, not clobbered
            let relays: Vec<PeerId> = bare
                .iter()
                .filter_map(|p| match p {
                    Protocol::P2p(id) => Some(id),
                    _ => None,
                })
                .collect();
            for r in relays {
                assert!(once.iter().any(|p| matches!(p, Protocol::P2p(id) if id == r)));
            }
        }
    }
}
