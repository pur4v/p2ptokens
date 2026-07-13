//! CLI entry point for the p2ptokens unified client. Thin wrapper over the
//! library's [`p2ptokens_client::run`]. Config precedence: CLI flags > env >
//! `p2ptokens.toml` > built-in defaults.

use anyhow::Result;
use clap::Parser;

use p2ptokens_client::{run, RunConfig};
use p2ptokens_shared::config::PlatformConfig;

#[derive(Parser)]
#[command(
    name = "p2ptokens",
    about = "p2ptokens unified client (seeder + leecher)"
)]
struct Args {
    /// path to a p2ptokens.toml config file (else ./p2ptokens.toml or defaults)
    #[arg(long)]
    config: Option<String>,
    /// coordinator base URL (overrides config `coordinator.url`)
    #[arg(long, env = "P2PTOKENS_COORDINATOR")]
    coordinator: Option<String>,
    /// local HTTP address (chat-completions endpoint + dashboard)
    #[arg(long)]
    http: Option<String>,
    /// libp2p listen multiaddr
    #[arg(long)]
    p2p_listen: Option<String>,
    /// data directory for the persisted identity (default: the per-OS data dir)
    #[arg(long)]
    data_dir: Option<String>,
    /// max concurrent jobs to serve
    #[arg(long)]
    capacity: Option<u32>,
    /// run as a public circuit-relay server (rendezvous for NAT'd peers)
    #[arg(long, default_value_t = false)]
    relay: bool,
    /// multiaddr of a relay to reserve a slot on (for reachability behind NAT)
    #[arg(long)]
    relay_addr: Option<String>,
    /// network namespace to join (overrides config `network.id`) — isolates the swarm
    #[arg(long)]
    network_id: Option<String>,
    /// private-network join secret (overrides config `network.join_secret`)
    #[arg(long)]
    join_secret: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    // Base config from file + env, then layer CLI overrides on top.
    let mut cfg = RunConfig::from_platform(PlatformConfig::load(args.config.as_deref()));
    if let Some(v) = args.coordinator {
        cfg.coordinator = v;
    }
    if let Some(v) = args.http {
        cfg.http = v;
    }
    if let Some(v) = args.p2p_listen {
        cfg.p2p_listen = v;
    }
    if args.data_dir.is_some() {
        cfg.data_dir = args.data_dir;
    }
    if let Some(v) = args.capacity {
        cfg.capacity = v;
    }
    if args.relay {
        cfg.relay = true;
    }
    if args.relay_addr.is_some() {
        cfg.relay_addr = args.relay_addr;
    }
    if let Some(v) = args.network_id {
        cfg.network_id = v;
    }
    if let Some(v) = args.join_secret {
        cfg.join_secret = Some(v);
    }

    run(cfg).await
}
