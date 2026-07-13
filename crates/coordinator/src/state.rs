//! Coordinator state, built for concurrency. Instead of one global lock, each
//! map is a sharded `DashMap` so independent requests touch independent shards
//! and scale across all CPU cores. The inference bytes never pass through here
//! (they are peer-to-peer), so the coordinator only does lightweight metadata
//! ops — a single multi-threaded process handles a very high request rate.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dashmap::{DashMap, DashSet};

use p2ptokens_shared::types::{Heartbeat, LedgerEntry, ModelCaps, ModelId};

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
/// Throughput (tok/s) at/above which a provider earns the full speed bonus.
const TPS_CAP: f64 = 100.0;

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
    if hb
        .peer_id
        .parse::<p2ptokens_shared::crypto::Peer>()
        .is_err()
    {
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
        if off
            .model
            .quant
            .as_ref()
            .is_some_and(|q| q.len() > MAX_STR_LEN)
        {
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

/// A live candidate provider gathered during matchmaking.
struct Cand {
    peer: String,
    addrs: Vec<String>,
    model: ModelId,
    newcomer: bool,
    tps: f64,
    caps_ok: bool,
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
    /// shared bearer secret for a PRIVATE network; when set, requests must
    /// present it. None = open network (public default).
    pub join_secret: Option<String>,
    match_counter: AtomicU64,
}

impl State {
    /// Build state. `allow_unsigned` = accept unsigned heartbeats (load-test
    /// only). `join_secret` = require this bearer on requests (private network).
    pub fn new(allow_unsigned: bool, join_secret: Option<String>) -> Self {
        Self {
            allow_unsigned,
            join_secret,
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
    /// LIVE, non-full candidates (never the consumer itself), lazily prune peers
    /// that vanished, then apply the capability filter with graceful fallback (if
    /// any candidate satisfies the required caps, keep only those; else keep all).
    fn sample_candidates(&self, req: &ModelId, consumer: &str, require: &ModelCaps) -> Vec<Cand> {
        let now = now_ms();
        let mut candidates: Vec<Cand> = Vec::new();
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
                        if let Some(off) = p.hb.offers.iter().find(|o| offer_matches(req, &o.model))
                        {
                            candidates.push(Cand {
                                caps_ok: off.caps.satisfies(require),
                                tps: off.tokens_per_sec,
                                newcomer: self.is_newcomer(&pid),
                                model: off.model.clone(),
                                addrs: p.hb.multiaddrs.clone(),
                                peer: pid.clone(),
                            });
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

        // Capability filter with fallback: prefer providers that meet the required
        // caps, but if none do, fall back to any provider serving the model.
        if candidates.iter().any(|c| c.caps_ok) {
            candidates.retain(|c| c.caps_ok);
        }
        candidates
    }

    /// Selection weight: reputation scaled by a throughput factor (1×–2×). Unknown
    /// throughput (0) is neutral so cold providers still get picked and accumulate
    /// samples.
    fn cand_weight(&self, c: &Cand) -> f64 {
        let rep = self.reputation_of(&c.peer).clamp(0.05, 1.0);
        let tps_factor = if c.tps <= 0.0 {
            1.0
        } else {
            1.0 + (c.tps.min(TPS_CAP) / TPS_CAP)
        };
        rep * tps_factor
    }

    /// Choose a provider for a model (no capability requirement). Convenience over
    /// [`Self::select_provider_req`].
    pub fn select_provider(
        &self,
        req: &ModelId,
        consumer: &str,
    ) -> Option<(String, Vec<String>, bool, ModelId)> {
        self.select_provider_req(req, consumer, &ModelCaps::default())
    }

    /// Choose a provider satisfying `require`. Roughly every AUDIT_EVERY-th match
    /// is reserved for a newcomer (optimistic unchoke); otherwise a
    /// throughput+reputation **weighted** pick spreads load across good, fast peers
    /// instead of funnelling everything onto the single top-ranked one.
    pub fn select_provider_req(
        &self,
        req: &ModelId,
        consumer: &str,
        require: &ModelCaps,
    ) -> Option<(String, Vec<String>, bool, ModelId)> {
        let mut candidates = self.sample_candidates(req, consumer, require);
        if candidates.is_empty() {
            return None;
        }

        let counter = self.match_counter.fetch_add(1, Ordering::Relaxed) + 1;

        if counter % AUDIT_EVERY == 0 {
            let newcomers: Vec<usize> = (0..candidates.len())
                .filter(|&i| candidates[i].newcomer)
                .collect();
            if !newcomers.is_empty() {
                let pick = newcomers[(counter as usize) % newcomers.len()];
                let c = candidates.swap_remove(pick);
                return Some((c.peer, c.addrs, true, c.model));
            }
        }

        let weights: Vec<f64> = candidates.iter().map(|c| self.cand_weight(c)).collect();
        let idx = weighted_index(&weights, counter);
        let c = candidates.swap_remove(idx);
        Some((c.peer, c.addrs, false, c.model))
    }

    /// Fan-out: up to `k` DISTINCT providers (no capability requirement).
    pub fn select_providers(
        &self,
        req: &ModelId,
        consumer: &str,
        k: usize,
    ) -> Vec<(String, Vec<String>, bool, ModelId)> {
        self.select_providers_req(req, consumer, k, &ModelCaps::default())
    }

    /// Fan-out: up to `k` DISTINCT providers satisfying `require`, best-first by
    /// throughput+reputation weight (racing/quorum/ensemble want the strongest K).
    pub fn select_providers_req(
        &self,
        req: &ModelId,
        consumer: &str,
        k: usize,
        require: &ModelCaps,
    ) -> Vec<(String, Vec<String>, bool, ModelId)> {
        let mut candidates = self.sample_candidates(req, consumer, require);
        candidates.sort_by(|a, b| {
            self.cand_weight(b)
                .partial_cmp(&self.cand_weight(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(k);
        candidates
            .into_iter()
            .map(|c| (c.peer, c.addrs, c.newcomer, c.model))
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

/// Deterministic PRNG (SplitMix64) — no external `rand` dependency.
fn splitmix64(seed: u64) -> u64 {
    let mut x = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Pick an index with probability proportional to `weights`, seeded by `seed`
/// (deterministic for a given seed → reproducible in tests; varies across matches
/// → spreads load).
fn weighted_index(weights: &[f64], seed: u64) -> usize {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return 0;
    }
    let r = (splitmix64(seed) as f64 / u64::MAX as f64) * total;
    let mut acc = 0.0;
    for (i, w) in weights.iter().enumerate() {
        acc += w;
        if r < acc {
            return i;
        }
    }
    weights.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use p2ptokens_shared::types::{ModelCaps, ModelOffer};

    fn model(name: &str) -> ModelId {
        ModelId {
            name: name.into(),
            quant: None,
        }
    }
    fn model_q(name: &str, q: &str) -> ModelId {
        ModelId {
            name: name.into(),
            quant: Some(q.into()),
        }
    }
    fn hb(peer: &str, m: &ModelId, cap: u32, inflight: u32) -> Heartbeat {
        Heartbeat {
            peer_id: peer.into(),
            multiaddrs: vec!["/ip4/203.0.113.7/tcp/40833".into()],
            offers: vec![ModelOffer {
                model: m.clone(),
                backend: "ollama".into(),
                ..Default::default()
            }],
            capacity: cap,
            in_flight: inflight,
            pubkey: String::new(),
            sig: String::new(),
        }
    }
    /// A heartbeat whose single offer advertises a serving throughput (tokens/sec).
    fn hb_tps(peer: &str, m: &ModelId, tps: f64) -> Heartbeat {
        let mut h = hb(peer, m, 4, 0);
        h.offers[0].tokens_per_sec = tps;
        h
    }
    fn caps(tools: bool) -> ModelCaps {
        ModelCaps {
            tools,
            ..Default::default()
        }
    }
    /// Insert a NON-newcomer ledger row with a fixed reputation (audits_total high).
    fn set_rep(st: &State, peer: &str, rep: f64) {
        let mut e = LedgerEntry::new(peer.to_string(), now_ms());
        e.reputation = rep;
        e.audits_total = NEWCOMER_AUDIT_THRESHOLD + 2;
        e.audits_passed = e.audits_total;
        st.ledger.insert(peer.to_string(), e);
    }

    // ---- matchmaking ----
    #[test]
    fn no_providers_yields_none() {
        let st = State::default();
        assert!(st.select_provider(&model("llama"), "consumer").is_none());
    }

    #[test]
    fn matches_live_provider_and_returns_concrete_model() {
        let st = State::default();
        st.register(hb("alice", &model_q("llama", "q4"), 4, 0));
        let m = st
            .select_provider(&model("llama"), "consumer")
            .expect("match");
        assert_eq!(m.0, "alice");
        assert_eq!(m.3.name, "llama");
        assert_eq!(m.3.quant.as_deref(), Some("q4")); // concrete quant handed back
    }

    #[test]
    fn excludes_the_consumer_itself() {
        let st = State::default();
        st.register(hb("alice", &model("llama"), 4, 0));
        assert!(st.select_provider(&model("llama"), "alice").is_none());
    }

    #[test]
    fn skips_full_providers() {
        let st = State::default();
        st.register(hb("alice", &model("llama"), 4, 4)); // in_flight == capacity
        assert!(st.select_provider(&model("llama"), "c").is_none());
    }

    #[test]
    fn stale_provider_is_skipped() {
        let st = State::default();
        st.register(hb("alice", &model("llama"), 4, 0));
        st.providers.get_mut("alice").unwrap().last_seen =
            now_ms().saturating_sub(HEARTBEAT_TTL_MS + 1_000);
        assert!(st.select_provider(&model("llama"), "c").is_none());
    }

    #[test]
    fn vanished_provider_is_pruned_from_index() {
        let st = State::default();
        st.register(hb("alice", &model("llama"), 4, 0));
        st.providers.remove("alice"); // peer gone from the directory but still indexed
        assert!(st.select_provider(&model("llama"), "c").is_none());
        // lazily pruned from the per-model index on the next match
        assert!(st
            .by_model
            .get("llama")
            .map(|s| s.is_empty())
            .unwrap_or(true));
    }

    #[test]
    fn quant_matching_rules() {
        let st = State::default();
        st.register(hb("alice", &model_q("llama", "q4"), 4, 0));
        assert!(st.select_provider(&model_q("llama", "q8"), "c").is_none()); // wrong quant
        assert!(st.select_provider(&model_q("llama", "q4"), "c").is_some()); // exact quant
        assert!(st.select_provider(&model("llama"), "c").is_some()); // no quant → any
        assert!(st.select_provider(&model("mistral"), "c").is_none()); // wrong name
    }

    #[test]
    fn weighted_selection_favors_reputation_but_spreads() {
        let st = State::default();
        st.register(hb("low", &model("llama"), 4, 0));
        st.register(hb("high", &model("llama"), 4, 0));
        set_rep(&st, "low", 0.30);
        set_rep(&st, "high", 0.95); // both non-newcomers → no audit slot interference
        let (mut hi, mut lo) = (0, 0);
        for _ in 0..300 {
            match st.select_provider(&model("llama"), "c").unwrap().0.as_str() {
                "high" => hi += 1,
                "low" => lo += 1,
                other => panic!("unexpected provider {other}"),
            }
        }
        // Reputation biases the draw toward the better peer …
        assert!(hi > lo, "higher reputation wins more often ({hi} vs {lo})");
        // … but weighting spreads load — the weaker peer is not starved.
        assert!(lo > 0, "lower reputation is not starved ({lo})");
    }

    #[test]
    fn weighted_selection_favors_faster_throughput() {
        let st = State::default();
        st.register(hb_tps("fast", &model("llama"), 100.0));
        st.register(hb_tps("slow", &model("llama"), 1.0));
        // Equal reputation → throughput is the only differentiator.
        set_rep(&st, "fast", 0.9);
        set_rep(&st, "slow", 0.9);
        let (mut f, mut s) = (0, 0);
        for _ in 0..300 {
            match st.select_provider(&model("llama"), "c").unwrap().0.as_str() {
                "fast" => f += 1,
                "slow" => s += 1,
                other => panic!("unexpected provider {other}"),
            }
        }
        assert!(f > s, "faster peer wins more often ({f} vs {s})");
        assert!(s > 0, "slow peer is not starved ({s})");
    }

    #[test]
    fn capability_filter_prefers_capable_then_falls_back() {
        // A tools-capable peer alongside a plain one; a tools request must route to
        // the capable peer every time.
        let st = State::default();
        let mut cap = hb("capable", &model("llama"), 4, 0);
        cap.offers[0].caps = caps(true);
        st.register(cap);
        st.register(hb("plain", &model("llama"), 4, 0));
        set_rep(&st, "capable", 0.5); // lower rep, but it's the only capable one
        set_rep(&st, "plain", 0.99);
        let require = caps(true);
        for _ in 0..30 {
            let m = st
                .select_provider_req(&model("llama"), "c", &require)
                .unwrap();
            assert_eq!(m.0, "capable", "tools request must hit the capable peer");
        }

        // When NO provider is capable, fall back to serving the model anyway rather
        // than returning nothing.
        let st2 = State::default();
        st2.register(hb("plain", &model("llama"), 4, 0));
        assert!(
            st2.select_provider_req(&model("llama"), "c", &require)
                .is_some(),
            "capability filter must fall back when nobody is capable"
        );
    }

    #[test]
    fn weighted_index_bounds_and_skew() {
        assert_eq!(weighted_index(&[], 1), 0, "empty → 0");
        assert_eq!(weighted_index(&[0.0, 0.0], 1), 0, "all-zero weights → 0");
        assert_eq!(weighted_index(&[1.0], 7), 0, "single element → 0");
        // Heavily skewed weights: index 1 should win the vast majority of seeds.
        let ones = (0..200u64)
            .filter(|&seed| weighted_index(&[0.01, 100.0], seed) == 1)
            .count();
        assert!(ones > 180, "dominant weight wins most draws ({ones}/200)");
    }

    #[test]
    fn audit_slot_routes_to_newcomer_on_fifth() {
        let st = State::default();
        st.register(hb("veteran", &model("llama"), 4, 0));
        set_rep(&st, "veteran", 0.99); // non-newcomer, top rep
        st.register(hb("newbie", &model("llama"), 4, 0)); // default ledger → newcomer
        for i in 1..=AUDIT_EVERY {
            let m = st.select_provider(&model("llama"), "c").unwrap();
            if i == AUDIT_EVERY {
                assert!(m.2, "every AUDIT_EVERY-th match is an audit/newcomer slot");
                assert_eq!(m.0, "newbie");
            } else {
                assert!(!m.2, "non-audit calls are not audit routes");
            }
        }
    }

    #[test]
    fn select_providers_distinct_and_clamped() {
        let st = State::default();
        for p in ["a", "b", "c"] {
            st.register(hb(p, &model("llama"), 4, 0));
        }
        let got = st.select_providers(&model("llama"), "consumer", 5); // ask 5, only 3 exist
        assert_eq!(got.len(), 3);
        let ids: std::collections::HashSet<_> = got.iter().map(|t| t.0.clone()).collect();
        assert_eq!(ids.len(), 3, "providers are distinct");
        assert_eq!(st.select_providers(&model("llama"), "consumer", 2).len(), 2);
    }

    // ---- ratio gate (the economy) ----
    #[test]
    fn consumer_allowed_grace_then_ratio_boundary() {
        let st = State::default();
        assert!(st.consumer_allowed("unknown")); // no ledger → allowed
        let mk = |consumed: u64, served: u64| {
            let mut e = LedgerEntry::new("x".into(), now_ms());
            e.consumed = consumed;
            e.served = served;
            e
        };
        // within grace, terrible ratio → allowed
        st.ledger
            .insert("g".into(), mk(NEWCOMER_GRACE_TOKENS - 1, 0));
        assert!(st.consumer_allowed("g"));
        // past grace, ratio < MIN → blocked
        st.ledger
            .insert("b".into(), mk(NEWCOMER_GRACE_TOKENS + 100, 10));
        assert!(!st.consumer_allowed("b"));
        // past grace, ratio exactly MIN_RATIO (0.5) → allowed (>=)
        st.ledger.insert("m".into(), mk(100_000, 50_000));
        assert!(st.consumer_allowed("m"));
        // past grace, healthy ratio → allowed
        st.ledger.insert("h".into(), mk(100_000, 100_000));
        assert!(st.consumer_allowed("h"));
    }

    // ---- input validation (unauthenticated heartbeat hardening) ----
    #[test]
    fn validate_heartbeat_accept_and_reject_paths() {
        let real =
            p2ptokens_shared::crypto::peer_id(&p2ptokens_shared::crypto::generate_identity())
                .to_string();
        let mut good = hb(&real, &model("llama"), 4, 0);
        assert!(validate_heartbeat(&good).is_ok());

        assert!(validate_heartbeat(&hb("not-a-peer", &model("llama"), 4, 0)).is_err());
        assert!(validate_heartbeat(&hb("", &model("llama"), 4, 0)).is_err());

        let mut over = good.clone();
        over.multiaddrs = (0..20).map(|i| format!("/ip4/1.2.3.4/tcp/{i}")).collect();
        assert!(validate_heartbeat(&over).is_err()); // too many addrs

        good.capacity = MAX_CAPACITY + 1;
        assert!(validate_heartbeat(&good).is_err()); // capacity ceiling
        good.capacity = 4;

        good.offers = (0..(MAX_OFFERS + 1))
            .map(|i| ModelOffer {
                model: model(&format!("m{i}")),
                backend: "ollama".into(),
                ..Default::default()
            })
            .collect();
        assert!(validate_heartbeat(&good).is_err()); // too many offers
    }

    // ---- job TTL sweep ----
    #[test]
    fn sweep_removes_only_expired_jobs() {
        let st = State::default();
        let job = |created: u64| Job {
            consumer: "c".into(),
            provider: "p".into(),
            model: model("llama"),
            audit: false,
            created,
        };
        st.jobs.insert("fresh".into(), job(now_ms()));
        st.jobs.insert(
            "old".into(),
            job(now_ms().saturating_sub(JOB_TTL_MS + 1_000)),
        );
        assert_eq!(st.sweep_jobs(JOB_TTL_MS), 1);
        assert!(st.jobs.contains_key("fresh"));
        assert!(!st.jobs.contains_key("old"));
    }
}
