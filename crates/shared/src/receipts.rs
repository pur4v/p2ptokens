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
