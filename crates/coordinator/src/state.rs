//! Coordinator state, built for concurrency. Instead of one global lock, each
//! map is a sharded `DashMap` so independent requests touch independent shards
//! and scale across all CPU cores. The inference bytes never pass through here
//! (they are peer-to-peer), so the coordinator only does lightweight metadata
//! ops — a single multi-threaded process handles a very high request rate.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::{DashMap, DashSet};

use p2ptokens_shared::types::{Heartbeat, LedgerEntry, ModelId};

/// Max live candidates gathered per match, and max index entries scanned to find
/// them. Both bound the per-match work to O(1) regardless of total swarm size.
const MATCH_SAMPLE: usize = 32;
const MATCH_MAX_SCAN: usize = 256;

/// Newcomer grace: a fresh consumer may draw down this many tokens before the
/// ratio gate applies (serve-first onboarding with grace, Q10).
pub const NEWCOMER_GRACE_TOKENS: u64 = 50_000;
/// Minimum served/consumed ratio required once past the grace allowance (Q16).
pub const MIN_RATIO: f64 = 0.5;
/// A provider heartbeat older than this is considered offline.
pub const HEARTBEAT_TTL_MS: u64 = 30_000;
/// Route roughly every Nth match through a newcomer/audit slot (Q11/Q15).
pub const AUDIT_EVERY: u64 = 5;
/// Providers with fewer than this many audits count as newcomers.
pub const NEWCOMER_AUDIT_THRESHOLD: u32 = 3;
/// Max providers a single fan-out request may claim (bounds jobs dialed per request).
pub const MAX_FANOUT: usize = 5;
/// A job unsettled for longer than this is swept (abandoned race losers / crashes).
pub const JOB_TTL_MS: u64 = 5 * 60 * 1000;

// --- Input bounds (heartbeats are unauthenticated, so cap everything an
// attacker could inflate to bloat memory or poison the per-model index). ---
/// Max dial addresses accepted per heartbeat.
const MAX_MULTIADDRS: usize = 8;
/// Max length of a single multiaddr string.
const MAX_ADDR_LEN: usize = 160;
/// Max model offers accepted per heartbeat.
const MAX_OFFERS: usize = 64;
/// Max length of a model name / quant / backend string.
const MAX_STR_LEN: usize = 128;
/// Ceiling on advertised concurrent-job capacity (sanity bound).
const MAX_CAPACITY: u32 = 4096;

/// Reject a heartbeat whose fields exceed sane bounds, or whose `peer_id` is not
/// a real libp2p PeerId. Cheap checks run BEFORE we touch any shared map, so a
/// malformed or hostile heartbeat can never allocate unbounded state or register
/// a garbage key in the index. Returns the reason on rejection.
pub fn validate_heartbeat(hb: &Heartbeat) -> Result<(), &'static str> {
    if hb.peer_id.is_empty() || hb.peer_id.len() > MAX_STR_LEN {
        return Err("peer_id length");
    }
    // must be a valid ed25519-derived PeerId, not arbitrary text
    if hb.peer_id.parse::<p2ptokens_shared::crypto::Peer>().is_err() {
        return Err("peer_id not a valid PeerId");
    }
    if hb.multiaddrs.len() > MAX_MULTIADDRS {
        return Err("too many multiaddrs");
    }
    if hb.multiaddrs.iter().any(|a| a.len() > MAX_ADDR_LEN) {
        return Err("multiaddr too long");
    }
    if hb.offers.len() > MAX_OFFERS {
        return Err("too many offers");
    }
    for off in &hb.offers {
        if off.model.name.is_empty() || off.model.name.len() > MAX_STR_LEN {
            return Err("model name length");
        }
        if off.model.quant.as_ref().is_some_and(|q| q.len() > MAX_STR_LEN) {
            return Err("quant length");
        }
        if off.backend.len() > MAX_STR_LEN {
            return Err("backend length");
        }
    }
    if hb.capacity > MAX_CAPACITY {
        return Err("capacity too high");
    }
    Ok(())
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub struct Provider {
    pub hb: Heartbeat,
    pub last_seen: u64,
}

pub struct Job {
    pub consumer: String,
    pub provider: String,
    pub model: ModelId,
    pub audit: bool,
    /// creation time (ms) — used by the TTL sweep to drop abandoned jobs.
    pub created: u64,
}

#[derive(Default)]
pub struct State {
    pub providers: DashMap<String, Provider>,
    pub ledger: DashMap<String, LedgerEntry>,
    pub jobs: DashMap<String, Job>,
    /// model name -> set of peer ids serving it (the "phonebook" that makes
    /// matchmaking O(1) instead of scanning every peer).
    pub by_model: DashMap<String, DashSet<String>>,
    /// accept heartbeats without a valid signature — INSECURE, for local load
    /// testing only (synthetic peers can't sign). Off by default.
    pub allow_unsigned: bool,
    match_counter: AtomicU64,
}

impl State {
    /// Build state, optionally accepting unsigned heartbeats (load-test only).
    pub fn new(allow_unsigned: bool) -> Self {
        Self {
            allow_unsigned,
            ..Default::default()
        }
    }

    /// Register (or refresh) a provider: update the live directory and the
    /// per-model index used for matchmaking.
    pub fn register(&self, hb: Heartbeat) {
        self.touch_ledger(&hb.peer_id);
        let peer = hb.peer_id.clone();
        for off in &hb.offers {
            self.by_model
                .entry(off.model.name.clone())
                .or_default()
                .insert(peer.clone());
        }
        self.providers.insert(
            peer,
            Provider {
                hb,
                last_seen: now_ms(),
            },
        );
    }

    /// Ensure a ledger row exists for a peer (cheap no-op if already present).
    pub fn touch_ledger(&self, peer: &str) {
        if !self.ledger.contains_key(peer) {
            self.ledger
                .entry(peer.to_string())
                .or_insert_with(|| LedgerEntry::new(peer.to_string(), now_ms()));
        }
    }

    pub fn reputation_of(&self, peer: &str) -> f64 {
        self.ledger.get(peer).map(|e| e.reputation).unwrap_or(1.0)
    }

    pub fn is_newcomer(&self, peer: &str) -> bool {
        self.ledger
            .get(peer)
            .map(|e| e.audits_total < NEWCOMER_AUDIT_THRESHOLD)
            .unwrap_or(true)
    }

    /// Is the consumer allowed to leech right now? Newcomers within the grace
    /// allowance always pass; past that, the ratio must clear MIN_RATIO.
    pub fn consumer_allowed(&self, consumer: &str) -> bool {
        match self.ledger.get(consumer) {
            None => true,
            Some(e) => e.consumed < NEWCOMER_GRACE_TOKENS || e.ratio() >= MIN_RATIO,
        }
    }

    /// O(1) sampling core: open the model's index page, take a bounded number of
    /// LIVE, non-full candidate providers (never the consumer itself), and lazily
    /// prune peers that have vanished. Shared by single- and multi-select.
    /// Returns (peer, addrs, concrete_model, is_newcomer) tuples.
    fn sample_candidates(
        &self,
        req: &ModelId,
        consumer: &str,
    ) -> Vec<(String, Vec<String>, ModelId, bool)> {
        let now = now_ms();
        let mut candidates: Vec<(String, Vec<String>, ModelId, bool)> = Vec::new();
        let mut stale: Vec<String> = Vec::new();

        {
            let Some(set) = self.by_model.get(&req.name) else {
                return candidates;
            };
            let mut scanned = 0usize;
            for entry in set.iter() {
                if candidates.len() >= MATCH_SAMPLE || scanned >= MATCH_MAX_SCAN {
                    break;
                }
                scanned += 1;
                let pid = entry.key().clone();
                if pid == consumer {
                    continue;
                }
                match self.providers.get(&pid) {
                    Some(p)
                        if now.saturating_sub(p.last_seen) < HEARTBEAT_TTL_MS
                            && p.hb.in_flight < p.hb.capacity =>
                    {
                        if let Some(off) = p.hb.offers.iter().find(|o| offer_matches(req, &o.model)) {
                            candidates.push((
                                pid.clone(),
                                p.hb.multiaddrs.clone(),
                                off.model.clone(),
                                self.is_newcomer(&pid),
                            ));
                        }
                    }
                    None => stale.push(pid), // peer gone → prune from index lazily
                    _ => {}
                }
            }
        }

        if !stale.is_empty() {
            if let Some(set) = self.by_model.get(&req.name) {
                for pid in stale {
                    set.remove(&pid);
                }
            }
        }
        candidates
    }

    /// Choose a provider for a model. Reputation-weighted, with roughly every
    /// AUDIT_EVERY-th match reserved for a newcomer (optimistic unchoke). Returns
    /// the provider plus the concrete model it will serve (with quant).
    pub fn select_provider(
        &self,
        req: &ModelId,
        consumer: &str,
    ) -> Option<(String, Vec<String>, bool, ModelId)> {
        let mut candidates = self.sample_candidates(req, consumer);
        if candidates.is_empty() {
            return None;
        }

        let counter = self.match_counter.fetch_add(1, Ordering::Relaxed) + 1;

        if counter % AUDIT_EVERY == 0 {
            let newcomers: Vec<usize> =
                (0..candidates.len()).filter(|&i| candidates[i].3).collect();
            if !newcomers.is_empty() {
                let pick = newcomers[(counter as usize) % newcomers.len()];
                let c = candidates.swap_remove(pick);
                return Some((c.0, c.1, true, c.2));
            }
        }

        candidates.sort_by(|a, b| {
            self.reputation_of(&b.0)
                .partial_cmp(&self.reputation_of(&a.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let c = candidates.swap_remove(0);
        Some((c.0, c.1, false, c.2))
    }

    /// Choose up to `k` DISTINCT providers for a model (client-side fan-out:
    /// racing / quorum / ensemble). Returns the top-k of the sample by
    /// reputation. Fewer than `k` (or empty) if the swarm can't supply that many.
    pub fn select_providers(
        &self,
        req: &ModelId,
        consumer: &str,
        k: usize,
    ) -> Vec<(String, Vec<String>, bool, ModelId)> {
        let mut candidates = self.sample_candidates(req, consumer);
        candidates.sort_by(|a, b| {
            self.reputation_of(&b.0)
                .partial_cmp(&self.reputation_of(&a.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(k);
        // reorder each tuple to (peer, addrs, is_newcomer, model) to match the
        // shape callers use (same as select_provider's return).
        candidates
            .into_iter()
            .map(|(peer, addrs, model, newcomer)| (peer, addrs, newcomer, model))
            .collect()
    }

    /// Remove jobs older than `max_age_ms` — a backstop so abandoned jobs (e.g.
    /// the peers a fan-out race drops, or a consumer that crashed mid-stream)
    /// don't accumulate in memory. Returns how many were swept.
    pub fn sweep_jobs(&self, max_age_ms: u64) -> usize {
        let now = now_ms();
        let expired: Vec<String> = self
            .jobs
            .iter()
            .filter(|e| now.saturating_sub(e.value().created) > max_age_ms)
            .map(|e| e.key().clone())
            .collect();
        let n = expired.len();
        for id in expired {
            self.jobs.remove(&id);
        }
        n
    }
}

/// A requested model matches an offer when the names are equal; if the consumer
/// specified a quant it must match too, otherwise any quant is fine (Q5).
fn offer_matches(req: &ModelId, offer: &ModelId) -> bool {
    if req.name != offer.name {
        return false;
    }
    match &req.quant {
        None => true,
        Some(q) => offer.quant.as_deref() == Some(q.as_str()),
    }
}
