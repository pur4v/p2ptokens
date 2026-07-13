//! Core domain types shared between the coordinator and the client daemon.
//!
//! The exchange unit in v1 is a standard streamed chat completion. A peer is
//! always both a potential seeder (serves completions from a local backend) and
//! a leecher (consumes completions from other peers) — the unified client.

use serde::{Deserialize, Serialize};

/// serde `skip_serializing_if` predicate: omit a bool field when it's false, so
/// a default value costs zero bytes on the wire.
fn is_false(b: &bool) -> bool {
    !*b
}

/// serde predicate: omit a zero float on the wire.
fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// Coarse model capabilities — advertised by a seeder and optionally required by
/// a consumer for capability-aware matching. All false by default (= unknown), so
/// they cost nothing on the wire and stay backward-compatible.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCaps {
    #[serde(default, skip_serializing_if = "is_false")]
    pub tools: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub vision: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub reasoning: bool,
}

impl ModelCaps {
    /// True if this offer satisfies every capability the request requires.
    pub fn satisfies(&self, req: &ModelCaps) -> bool {
        (!req.tools || self.tools)
            && (!req.vision || self.vision)
            && (!req.reasoning || self.reasoning)
    }
    /// No capability set/required.
    pub fn is_empty(&self) -> bool {
        !self.tools && !self.vision && !self.reasoning
    }
}

/// A model identity as advertised and matched on. Backend-agnostic (Q5/Q13).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelId {
    /// e.g. "llama3.1:8b", "gpt-4o", "claude-3-5-sonnet-latest"
    pub name: String,
    /// quantization / precision tag when meaningful, e.g. "q4_K_M".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quant: Option<String>,
}

impl ModelId {
    /// Canonical string key used for matching.
    pub fn key(&self) -> String {
        match &self.quant {
            Some(q) if !q.is_empty() => format!("{}@{}", self.name, q),
            _ => self.name.clone(),
        }
    }
}

/// What a seeder can serve. The backend kind is provider-internal and never
/// exposed to consumers — matching is purely by model name.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelOffer {
    pub model: ModelId,
    /// "ollama" | "endpoint" | "claude"
    pub backend: String,
    /// coarse capabilities of this model (tools/vision/reasoning). Empty = unknown.
    #[serde(default, skip_serializing_if = "ModelCaps::is_empty")]
    pub caps: ModelCaps,
    /// serving throughput estimate (tokens/sec, EMA) the seeder reports so the
    /// coordinator can prefer faster peers. 0 = unknown (neutral).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub tokens_per_sec: f64,
}

/// Heartbeat a seeder sends to the coordinator to stay in the live directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub peer_id: String,
    /// Bare libp2p dial addresses — WITHOUT a trailing `/p2p/<peer_id>` (that id
    /// is this heartbeat's `peer_id`; repeating it in every address just wastes
    /// ~52 bytes each). Consumers re-attach it when dialing.
    pub multiaddrs: Vec<String>,
    pub offers: Vec<ModelOffer>,
    /// max concurrent jobs this seeder will accept right now
    pub capacity: u32,
    pub in_flight: u32,
    /// sender's public key (b64 protobuf) — proves the heartbeat came from the
    /// holder of `peer_id`. Verified by the coordinator (see shared `auth`).
    #[serde(default)]
    pub pubkey: String,
    /// signature over the canonical heartbeat body with the peer's key.
    #[serde(default)]
    pub sig: String,
}

/// A consumer's request to the coordinator to be matched with a seeder.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchRequest {
    pub consumer: String,
    pub model: ModelId,
    /// capabilities the consumer requires (e.g. `tools`). Empty = no requirement;
    /// if no capable provider exists the coordinator falls back to any provider.
    #[serde(default, skip_serializing_if = "ModelCaps::is_empty")]
    pub require: ModelCaps,
}

/// The coordinator's answer: a seeder to dial, plus a signed job grant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub job_id: String,
    pub provider: String,
    /// Bare dial addresses without the trailing `/p2p/<provider>` component —
    /// that id is already in `provider`, so sending it inside every address too
    /// duplicates ~52 bytes each. The consumer re-attaches it before dialing
    /// (client `ensure_peer`); libp2p still authenticates the remote against
    /// `provider` via `DialOpts::peer_id`, so a tampered address cannot redirect
    /// the dial to a different peer — the Noise handshake fails on mismatch.
    pub multiaddrs: Vec<String>,
    pub model: ModelId,
    /// true if routed as a newcomer/audit slot (optimistic unchoke, Q11/Q15).
    /// Omitted on the wire when false (the common case) to save bytes.
    #[serde(default, skip_serializing_if = "is_false")]
    pub audit: bool,
}

/// Per-identity accounting the coordinator maintains (the barter ratio, Q16).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub peer_id: String,
    /// cumulative output tokens SERVED to others (upload)
    pub served: u64,
    /// cumulative output tokens CONSUMED from others (download)
    pub consumed: u64,
    /// rolling reputation score in [0,1] from challenge-audits
    pub reputation: f64,
    pub audits_passed: u32,
    pub audits_total: u32,
    pub first_seen: u64,
}

impl LedgerEntry {
    pub fn new(peer_id: String, now: u64) -> Self {
        Self {
            peer_id,
            served: 0,
            consumed: 0,
            reputation: 1.0,
            audits_passed: 0,
            audits_total: 0,
            first_seen: now,
        }
    }

    /// Upload/download ratio. Newcomers with no consumption are treated healthy.
    pub fn ratio(&self) -> f64 {
        if self.consumed == 0 {
            f64::INFINITY
        } else {
            self.served as f64 / self.consumed as f64
        }
    }
}

#[cfg(test)]
mod type_tests {
    use super::*;

    #[test]
    fn model_key_formats() {
        assert_eq!(
            ModelId {
                name: "llama".into(),
                quant: None
            }
            .key(),
            "llama"
        );
        assert_eq!(
            ModelId {
                name: "llama".into(),
                quant: Some("q4".into())
            }
            .key(),
            "llama@q4"
        );
        // empty quant is treated as no quant
        assert_eq!(
            ModelId {
                name: "llama".into(),
                quant: Some(String::new())
            }
            .key(),
            "llama"
        );
    }

    #[test]
    fn ledger_ratio_math() {
        let mut e = LedgerEntry::new("p".into(), 0);
        assert!(e.ratio().is_infinite()); // nothing consumed yet
        e.consumed = 100;
        e.served = 50;
        assert!((e.ratio() - 0.5).abs() < 1e-9);
    }
}

// ---- Chat completion surface (the subset we support) ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChatCompletionParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// A streamed piece of generated output from a backend adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionDelta {
    /// incremental text produced since the previous delta
    pub text: String,
    /// cumulative output token count so far (adapter's best estimate)
    pub cumulative_tokens: u64,
    pub done: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}
