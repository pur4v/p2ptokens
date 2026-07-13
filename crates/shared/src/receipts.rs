//! Co-receipts: the trustless metering mechanism (design Q18-A).
//!
//! Work streams in chunks. After each chunk the CONSUMER signs a receipt
//! acknowledging the cumulative output-token count received. The provider only
//! continues once it holds the receipt for the previous chunk. At settlement the
//! provider submits the highest-seq consumer-signed receipt to the coordinator,
//! which verifies the signature and credits/debits the ratio.
//!
//! Consequence: neither side can lie by more than one chunk.

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};

use crate::crypto;
use libp2p_identity::{PeerId, PublicKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptBody {
    pub job_id: String,
    pub consumer: String, // consumer peer id
    pub provider: String, // provider peer id
    pub model: String,    // ModelId::key()
    pub seq: u64,
    pub cumulative_tokens: u64,
    pub ts: u64, // consumer-provided timestamp (ms)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedReceipt {
    pub body: ReceiptBody,
    /// consumer's signature over the canonical body — the binding acknowledgement
    pub consumer_sig: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_sig: Option<String>,
}

impl ReceiptBody {
    /// Deterministic byte encoding for signing/verification.
    pub fn canonical(&self) -> Vec<u8> {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.job_id,
            self.consumer,
            self.provider,
            self.model,
            self.seq,
            self.cumulative_tokens,
            self.ts
        )
        .into_bytes()
    }
}

pub fn sign_receipt(kp: &crypto::Identity, body: &ReceiptBody) -> Result<String> {
    Ok(B64.encode(crypto::sign(kp, &body.canonical())?))
}

/// Verify the consumer signature on a receipt, given the consumer's public key.
pub fn verify_receipt(body: &ReceiptBody, sig_b64: &str, consumer_pk: &PublicKey) -> bool {
    let Ok(sig) = B64.decode(sig_b64) else {
        return false;
    };
    let Ok(consumer_peer) = body.consumer.parse::<PeerId>() else {
        return false;
    };
    crypto::verify(&consumer_peer, consumer_pk, &body.canonical(), &sig)
}

/// Full settlement validation: signature verifies and participants/ids match.
pub fn verify_settlement(
    r: &SignedReceipt,
    consumer_pk: &PublicKey,
    expect_job: &str,
    expect_consumer: &str,
    expect_provider: &str,
) -> bool {
    r.body.job_id == expect_job
        && r.body.consumer == expect_consumer
        && r.body.provider == expect_provider
        && verify_receipt(&r.body, &r.consumer_sig, consumer_pk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto;

    /// Build a validly consumer-signed receipt + the consumer's public key.
    fn signed(job: &str, provider: &str) -> (SignedReceipt, crypto::PubKey, String) {
        let kp = crypto::generate_identity();
        let consumer = crypto::peer_id(&kp).to_string();
        let body = ReceiptBody {
            job_id: job.into(),
            consumer: consumer.clone(),
            provider: provider.into(),
            model: "llama".into(),
            seq: 1,
            cumulative_tokens: 100,
            ts: 0,
        };
        let sig = sign_receipt(&kp, &body).unwrap();
        let r = SignedReceipt {
            body,
            consumer_sig: sig,
            provider_sig: None,
        };
        (r, kp.public(), consumer)
    }

    #[test]
    fn valid_settlement_verifies() {
        let (r, pk, consumer) = signed("job1", "prov1");
        assert!(verify_settlement(&r, &pk, "job1", &consumer, "prov1"));
    }

    #[test]
    fn tampered_token_count_fails() {
        let (mut r, pk, consumer) = signed("job1", "prov1");
        r.body.cumulative_tokens = 999_999; // change a signed field after signing
        assert!(!verify_settlement(&r, &pk, "job1", &consumer, "prov1"));
    }

    #[test]
    fn mismatched_ids_fail() {
        let (r, pk, consumer) = signed("job1", "prov1");
        assert!(!verify_settlement(&r, &pk, "WRONG", &consumer, "prov1")); // job
        assert!(!verify_settlement(&r, &pk, "job1", "WRONG", "prov1")); // consumer
        assert!(!verify_settlement(&r, &pk, "job1", &consumer, "WRONG")); // provider
    }

    #[test]
    fn wrong_pubkey_fails() {
        let (r, _pk, consumer) = signed("job1", "prov1");
        let other = crypto::generate_identity().public(); // not the signer
        assert!(!verify_settlement(&r, &other, "job1", &consumer, "prov1"));
    }

    #[test]
    fn garbage_signature_fails() {
        let (mut r, pk, consumer) = signed("job1", "prov1");
        r.consumer_sig = "not-base64!!".into();
        assert!(!verify_settlement(&r, &pk, "job1", &consumer, "prov1"));
    }
}
