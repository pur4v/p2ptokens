//! Shared runtime context for the unified client (seeder + leecher).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use libp2p::PeerId;

use p2ptokens_shared::config::BrandConfig;
use p2ptokens_shared::crypto;
use p2ptokens_shared::types::ModelOffer;

use crate::adapters::Adapter;
use crate::coordinator_client::CoordinatorClient;
use crate::node::NodeHandle;

/// How to serve a given model key: which adapter, and the backend model name.
pub struct ModelServe {
    pub adapter: usize,
    pub name: String,
}

pub struct Ctx {
    pub keypair: crypto::Identity,
    pub pubkey_b64: String,
    pub local_peer: PeerId,
    pub node: NodeHandle,
    pub coord: CoordinatorClient,
    pub adapters: Vec<Adapter>,
    /// model key -> serving info (seeder side)
    pub model_index: HashMap<String, ModelServe>,
    /// what we advertise to the coordinator
    pub offers: Vec<ModelOffer>,
    pub capacity: u32,
    pub in_flight: Arc<AtomicU32>,
    /// serving throughput EMA (tokens/sec) as `f64::to_bits`, updated by the
    /// seeder and reported in heartbeats so the coordinator can prefer fast peers.
    pub tps_ema: Arc<AtomicU64>,
    /// network namespace (scopes the libp2p protocol → swarm isolation)
    pub network_id: String,
    /// white-label branding served to the dashboard via `/api/config`
    pub brand: BrandConfig,
    /// local SQLite chat history (threads + messages)
    pub threads: Arc<crate::threads::ThreadStore>,
}

pub type SharedCtx = Arc<Ctx>;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
