//! HTTP DTOs for the coordinator's tracker API, shared so client and coordinator
//! agree on the wire shape.

use serde::{Deserialize, Serialize};

use crate::receipts::SignedReceipt;

/// Provider submits the final consumer-signed receipt to settle a job and update
/// the ratio ledger (Q16/Q18).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleRequest {
    pub receipt: SignedReceipt,
    /// consumer's public key (b64 protobuf) so the coordinator can verify the sig
    pub consumer_pubkey: String,
    /// whether the job streamed to completion (feeds audit pass/fail)
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettleResponse {
    pub accepted: bool,
    pub provider_served_total: u64,
    pub consumer_consumed_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Request to `POST /match_many` — ask for up to `count` DISTINCT providers for
/// the same model, used by client-side fan-out (racing / quorum / ensemble).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchManyRequest {
    pub consumer: String,
    pub model: crate::types::ModelId,
    /// how many distinct providers to return (coordinator clamps to a max).
    #[serde(default)]
    pub count: u32,
}

/// Response to `POST /match_many`. `Matched.matches` may be shorter than the
/// requested `count` (or empty) if fewer live providers exist.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "s")]
pub enum MatchManyResponse {
    #[serde(rename = "m")]
    Matched { matches: Vec<crate::types::Match> },
    #[serde(rename = "r")]
    RatioExceeded,
}

/// Response to `POST /match` — either a match or a code for why none was given.
///
/// Wire form is tagged by a single-character `"s"` field (`m`/`n`/`r`) and the
/// failure variants carry NO human text: the reason string was the same fixed
/// sentence every time, so we send only the code and let the client render the
/// message (the words live in the UI, not on the wire).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "s")]
pub enum MatchResponse {
    #[serde(rename = "m")]
    Matched(crate::types::Match),
    /// no live seeder for the requested model
    #[serde(rename = "n")]
    NoProvider,
    /// consumer must serve more to restore its ratio before leeching (Q16)
    #[serde(rename = "r")]
    RatioExceeded,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Match, ModelId};

    fn a_match(audit: bool) -> Match {
        Match {
            job_id: "j".into(),
            provider: "p".into(),
            multiaddrs: vec!["/ip4/203.0.113.7/tcp/40833".into()],
            model: ModelId {
                name: "llama".into(),
                quant: None,
            },
            audit,
        }
    }

    #[test]
    fn match_response_single_char_tags_and_omits_default_audit() {
        let s = serde_json::to_string(&MatchResponse::Matched(a_match(false))).unwrap();
        assert!(s.contains(r#""s":"m""#), "tagged with s=m: {s}");
        assert!(
            !s.contains("audit"),
            "audit=false omitted from the wire: {s}"
        );
        assert_eq!(
            serde_json::to_string(&MatchResponse::NoProvider).unwrap(),
            r#"{"s":"n"}"#
        );
        assert_eq!(
            serde_json::to_string(&MatchResponse::RatioExceeded).unwrap(),
            r#"{"s":"r"}"#
        );
    }

    #[test]
    fn match_response_roundtrips_with_audit_true() {
        let s = serde_json::to_string(&MatchResponse::Matched(a_match(true))).unwrap();
        assert!(s.contains(r#""audit":true"#));
        match serde_json::from_str::<MatchResponse>(&s).unwrap() {
            MatchResponse::Matched(m) => {
                assert_eq!(m.job_id, "j");
                assert!(m.audit);
            }
            _ => panic!("expected Matched"),
        }
    }
}
