//! CLI entry point for the p2ptokens unified client. Thin wrapper over the
//! library's [`p2ptokens_client::run`].

use anyhow::Result;
use clap::Parser;

use p2ptokens_client::{run, RunConfig};

#[derive(Parser)]
#[command(name = "p2ptokens", about = "p2ptokens unified client (seeder + leecher)")]
struct Args {
    /// coordinator base URL (env: P2PTOKENS_COORDINATOR)
    #[arg(
        long,
        env = "P2PTOKENS_COORDINATOR",
        default_value = "https://coordinator.p2ptokens.com"
    )]
    coordinator: String,
    /// local HTTP address (chat-completions endpoint + dashboard)
    #[arg(long, default_value = "127.0.0.1:8080")]
    http: String,
    /// libp2p listen multiaddr
    #[arg(long, default_value = "/ip4/127.0.0.1/tcp/0")]
    p2p_listen: String,
    /// data directory for the persisted identity (default: the per-OS data dir)
    #[arg(long)]
    data_dir: Option<String>,
    /// max concurrent jobs to serve
    #[arg(long, default_value_t = 4)]
    capacity: u32,
    /// run as a public circuit-relay server (rendezvous for NAT'd peers)
    #[arg(long, default_value_t = false)]
    relay: bool,
    /// multiaddr of a relay to reserve a slot on (for reachability behind NAT)
    #[arg(long)]
    relay_addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let args = Args::parse();
    run(RunConfig {
        coordinator: args.coordinator,
        http: args.http,
        p2p_listen: args.p2p_listen,
        data_dir: args.data_dir,
        capacity: args.capacity,
        relay: args.relay,
        relay_addr: args.relay_addr,
    })
    .await
}
