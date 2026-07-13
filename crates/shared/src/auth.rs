//! Heartbeat authentication.
//!
//! Heartbeats register a peer (and its offers) in the coordinator's live
//! directory and per-model index. Without a signature, anyone could register
//! under someone else's `peer_id` — poisoning the index, spoofing capacity, or
//! flooding fake peers. So each heartbeat is signed by the peer's key and the
//! coordinator verifies the key hashes to the claimed `peer_id`, exactly like
//! co-receipts ([`crate::receipts`]).

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use libp2p_identity::PeerId;
use serde::Serialize;

use crate::crypto;
use crate::types::{Heartbeat, ModelOffer};

/// The signed portion of a heartbeat — everything except the credentials
/// themselves. Serialized deterministically (fixed struct field order, no maps)
/// so signer and verifier produce identical bytes.
#[derive(Serialize)]
struct SignedBody<'a> {
    peer_id: &'a str,
    multiaddrs: &'a [String],
    offers: &'a [ModelOffer],
    capacity: u32,
    in_flight: u32,
}

fn canonical(hb: &Heartbeat) -> Vec<u8> {
    serde_json::to_vec(&SignedBody {
        peer_id: &hb.peer_id,
        multiaddrs: &hb.multiaddrs,
        offers: &hb.offers,
        capacity: hb.capacity,
        in_flight: hb.in_flight,
    })
    .unwrap_or_default()
}

/// Sign a heartbeat in place: attach the peer's public key and a signature over
/// the canonical body, proving the sender controls `peer_id`.
pub fn sign_heartbeat(kp: &crypto::Identity, hb: &mut Heartbeat) -> Result<()> {
    hb.pubkey = crypto::export_pubkey(&kp.public());
    hb.sig = B64.encode(crypto::sign(kp, &canonical(hb))?);
    Ok(())
}

/// Verify a heartbeat's signature and that the presented key hashes to the
/// claimed `peer_id`. Returns false on any malformed field.
pub fn verify_heartbeat(hb: &Heartbeat) -> bool {
    let Ok(pk) = crypto::import_pubkey(&hb.pubkey) else {
        return false;
    };
    let Ok(sig) = B64.decode(&hb.sig) else {
        return false;
    };
    let Ok(peer) = hb.peer_id.parse::<PeerId>() else {
        return false;
    };
    crypto::verify(&peer, &pk, &canonical(hb), &sig)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModelId, ModelOffer};

    fn sample() -> Heartbeat {
        Heartbeat {
            peer_id: String::new(),
            multiaddrs: vec!["/ip4/203.0.113.7/tcp/40833".into()],
            offers: vec![ModelOffer {
                model: ModelId {
                    name: "llama3.1:8b".into(),
                    quant: None,
                },
                backend: "ollama".into(),
                ..Default::default()
            }],
            capacity: 4,
            in_flight: 0,
            pubkey: String::new(),
            sig: String::new(),
        }
    }

    #[test]
    fn signed_heartbeat_verifies() {
        let kp = crypto::generate_identity();
        let mut hb = sample();
        hb.peer_id = crypto::peer_id(&kp).to_string();
        sign_heartbeat(&kp, &mut hb).unwrap();
        assert!(verify_heartbeat(&hb));
    }

    #[test]
    fn tampered_body_fails() {
        let kp = crypto::generate_identity();
        let mut hb = sample();
        hb.peer_id = crypto::peer_id(&kp).to_string();
        sign_heartbeat(&kp, &mut hb).unwrap();
        hb.capacity = 9999; // change a signed field after signing
        assert!(!verify_heartbeat(&hb));
    }

    #[test]
    fn survives_json_roundtrip_with_nonzero_tps() {
        // A heartbeat that has served traffic carries a non-zero tokens_per_sec and
        // capabilities on its offers. It must still verify after crossing the wire
        // (sign -> JSON -> parse -> verify), like a real provider heartbeat.
        let kp = crypto::generate_identity();
        let mut hb = sample();
        hb.peer_id = crypto::peer_id(&kp).to_string();
        hb.offers[0].tokens_per_sec = 10.456355223567767;
        hb.offers[0].caps = crate::types::ModelCaps {
            vision: true,
            ..Default::default()
        };
        sign_heartbeat(&kp, &mut hb).unwrap();
        let json = serde_json::to_string(&hb).unwrap();
        let received: Heartbeat = serde_json::from_str(&json).unwrap();
        assert!(
            verify_heartbeat(&received),
            "heartbeat must verify post-wire"
        );
    }

    #[test]
    fn wrong_identity_fails() {
        // sign with one key but claim a different peer_id -> rejected
        let signer = crypto::generate_identity();
        let victim = crypto::generate_identity();
        let mut hb = sample();
        hb.peer_id = crypto::peer_id(&victim).to_string();
        sign_heartbeat(&signer, &mut hb).unwrap();
        assert!(!verify_heartbeat(&hb));
    }
}
