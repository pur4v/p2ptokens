//! p2ptokens unified client as a library: a single daemon that is simultaneously
//! a seeder (serves completions from local backends) and a leecher (consumes
//! completions from peers), exposing a local drop-in chat-completions endpoint +
//! dashboard. Exposed as [`run`] so it can be embedded (e.g., in the desktop app)
//! as well as driven by the CLI binary.

mod adapters;
mod config;
mod coordinator_client;
mod ctx;
mod http;
mod leecher;
mod node;
mod seeder;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use p2ptokens_shared::crypto;
use p2ptokens_shared::types::{Heartbeat, ModelOffer};

use adapters::{Adapter, ClaudeAdapter, CompatAdapter, OllamaAdapter};
use config::{load_or_create_identity, BackendConfig};
use coordinator_client::CoordinatorClient;
use ctx::{Ctx, ModelServe};

/// Runtime configuration for the client daemon.
#[derive(Debug, Clone)]
pub struct RunConfig {
    /// coordinator base URL
    pub coordinator: String,
    /// local HTTP address (chat-completions endpoint + dashboard)
    pub http: String,
    /// libp2p listen multiaddr
    pub p2p_listen: String,
    /// data directory for the persisted identity (None = per-OS default)
    pub data_dir: Option<String>,
    /// max concurrent jobs to serve
    pub capacity: u32,
    /// run as a public circuit-relay server
    pub relay: bool,
    /// multiaddr of a relay to reserve a slot on
    pub relay_addr: Option<String>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            // Ship pointing at the hosted coordinator; override with
            // P2PTOKENS_COORDINATOR (e.g. http://127.0.0.1:4000 for local dev).
            coordinator: std::env::var("P2PTOKENS_COORDINATOR")
                .unwrap_or_else(|_| "https://coordinator.p2ptokens.com".to_string()),
            http: "127.0.0.1:8080".to_string(),
            p2p_listen: "/ip4/0.0.0.0/tcp/0".to_string(),
            data_dir: None,
            capacity: 4,
            relay: false,
            relay_addr: None,
        }
    }
}

/// Resolve the data directory (explicit override, else the per-OS data dir).
pub fn resolve_data_dir(data_dir: &Option<String>) -> PathBuf {
    data_dir
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("p2ptokens")
        })
}

/// Start the unified client and serve until the process exits.
pub async fn run(cfg: RunConfig) -> Result<()> {
    let data_dir = resolve_data_dir(&cfg.data_dir);
    let keypair = load_or_create_identity(&data_dir)?;
    let local_peer = crypto::peer_id(&keypair);
    let pubkey_b64 = crypto::export_pubkey(&keypair.public());

    let relay_addr = match &cfg.relay_addr {
        Some(a) => Some(a.parse()?),
        None => None,
    };
    let (node, incoming) = node::start(
        keypair.clone(),
        node::NodeConfig {
            listen: cfg.p2p_listen.parse()?,
            relay: cfg.relay,
            relay_addr,
        },
    )
    .await?;

    // Assemble backends from the environment (BYO-credentials, Q13).
    let bc = BackendConfig::from_env();
    let mut adapters: Vec<Adapter> = Vec::new();
    if bc.ollama {
        adapters.push(Adapter::Ollama(OllamaAdapter::new(bc.ollama_url.clone())));
    }
    if let Some(url) = &bc.endpoint_url {
        if !bc.endpoint_models.is_empty() {
            adapters.push(Adapter::Compat(CompatAdapter::new(
                url.clone(),
                bc.endpoint_key.clone(),
                bc.endpoint_models.clone(),
            )));
        }
    }
    if let Some(k) = &bc.claude_key {
        if !bc.claude_models.is_empty() {
            adapters.push(Adapter::Claude(ClaudeAdapter::new(
                k.clone(),
                bc.claude_models.clone(),
                None,
            )));
        }
    }

    // Discover what each backend can serve; build the advertised offer set and
    // the model-key -> backend routing index.
    let mut offers: Vec<ModelOffer> = Vec::new();
    let mut model_index: HashMap<String, ModelServe> = HashMap::new();
    for (i, a) in adapters.iter().enumerate() {
        match a.list_models().await {
            Ok(models) => {
                for m in models {
                    let key = m.key();
                    offers.push(ModelOffer {
                        model: m.clone(),
                        backend: a.backend().to_string(),
                    });
                    model_index.entry(key).or_insert(ModelServe {
                        adapter: i,
                        name: m.name.clone(),
                    });
                }
            }
            Err(e) => tracing::warn!("backend {} list_models failed: {e:#}", a.backend()),
        }
    }
    tracing::info!(
        "serving {} model(s) across {} backend(s)",
        offers.len(),
        adapters.len()
    );

    let ctx = Arc::new(Ctx {
        keypair,
        pubkey_b64,
        local_peer,
        node,
        coord: CoordinatorClient::new(cfg.coordinator.clone()),
        adapters,
        model_index,
        offers,
        capacity: cfg.capacity,
        in_flight: Arc::new(AtomicU32::new(0)),
    });

    // Seeder: serve inbound completion streams.
    tokio::spawn(seeder::serve(ctx.clone(), incoming));

    // Heartbeat: keep our offers + dial addresses fresh in the coordinator.
    {
        let ctx = ctx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(800)).await;
            loop {
                // Advertise BARE dial addresses (no trailing `/p2p/<self>`): our
                // id already travels in `peer_id`, so appending it to every
                // address just duplicates ~52 bytes each. Consumers re-attach it
                // when dialing. (Relayed addrs keep their intermediate
                // `/p2p/<relay>/p2p-circuit` — only our own trailing id is
                // absent because `listen_addrs()` never contains it.)
                let multiaddrs: Vec<String> = ctx
                    .node
                    .listen_addrs()
                    .iter()
                    .map(|a| a.to_string())
                    .collect();
                let mut hb = Heartbeat {
                    peer_id: ctx.local_peer.to_string(),
                    multiaddrs,
                    offers: ctx.offers.clone(),
                    capacity: ctx.capacity,
                    in_flight: ctx.in_flight.load(Ordering::SeqCst),
                    pubkey: String::new(),
                    sig: String::new(),
                };
                // Sign so the coordinator can prove this heartbeat came from us
                // (not someone registering under our peer id).
                if let Err(e) = p2ptokens_shared::auth::sign_heartbeat(&ctx.keypair, &mut hb) {
                    tracing::debug!("sign heartbeat failed: {e:#}");
                } else if let Err(e) = ctx.coord.heartbeat(&hb).await {
                    tracing::debug!("heartbeat failed: {e:#}");
                }
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
    }

    let listener = tokio::net::TcpListener::bind(&cfg.http).await?;
    tracing::info!(
        "client dashboard: http://{}  |  peer: {}",
        cfg.http,
        local_peer
    );
    axum::serve(listener, http::router(ctx)).await?;
    Ok(())
}
