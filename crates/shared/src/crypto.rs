//! Identity + signing. The libp2p ed25519 `Keypair` is the single identity for
//! BOTH transport (PeerId) and application-level signatures (co-receipts).
//! One keypair, one identity (design Q9/Q12).

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use libp2p_identity::{Keypair, PeerId, PublicKey};

pub use libp2p_identity::{Keypair as Identity, PeerId as Peer, PublicKey as PubKey};

/// Generate a fresh anonymous ed25519 identity.
pub fn generate_identity() -> Keypair {
    Keypair::generate_ed25519()
}

/// Serialize a keypair (secret included) to a base64 string for on-disk storage.
pub fn export_secret(kp: &Keypair) -> Result<String> {
    let bytes = kp
        .to_protobuf_encoding()
        .context("encode keypair")?;
    Ok(B64.encode(bytes))
}

/// Restore a keypair from its base64 protobuf encoding.
pub fn import_secret(s: &str) -> Result<Keypair> {
    let bytes = B64.decode(s.trim()).context("decode keypair base64")?;
    Keypair::from_protobuf_encoding(&bytes).context("decode keypair protobuf")
}

pub fn peer_id(kp: &Keypair) -> PeerId {
    kp.public().to_peer_id()
}

/// Base64 protobuf encoding of a public key, carried in the wire handshake and
/// in coordinator settlement so receipts can be verified without key exchange.
pub fn export_pubkey(pk: &PublicKey) -> String {
    B64.encode(pk.encode_protobuf())
}

pub fn import_pubkey(s: &str) -> Result<PublicKey> {
    let bytes = B64.decode(s.trim()).context("decode pubkey base64")?;
    PublicKey::try_decode_protobuf(&bytes).context("decode pubkey protobuf")
}

pub fn sign(kp: &Keypair, data: &[u8]) -> Result<Vec<u8>> {
    kp.sign(data).context("sign")
}

/// Verify a signature and confirm the signer's public key hashes to `expected`.
pub fn verify(expected: &PeerId, pubkey: &PublicKey, data: &[u8], sig: &[u8]) -> bool {
    pubkey.to_peer_id() == *expected && pubkey.verify(data, sig)
}
