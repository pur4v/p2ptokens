//! p2ptokens desktop app (Tauri): a proper double-click application that embeds
//! the P2P daemon in-process and shows the dashboard in a native window. Fully
//! peer-to-peer — no servers in the inference data path.
//!
//! The daemon serves the real dashboard + APIs on a local port; the window first
//! shows a bundled loading page (dist/index.html) that hands over to that port
//! once the daemon is up (keeping all window creation on the main thread).

// Hide the extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Local port the embedded daemon serves the dashboard + `/v1` API on.
const DASHBOARD_ADDR: &str = "127.0.0.1:8787";

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Start the P2P daemon on its own thread + Tokio runtime, so it stays
    // independent of Tauri's main-thread UI event loop.
    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");
        rt.block_on(async {
            let cfg = p2ptokens_client::RunConfig {
                http: DASHBOARD_ADDR.to_string(),
                // reachable across the internet; relay/DCUtR handle NAT
                p2p_listen: "/ip4/0.0.0.0/tcp/0".to_string(),
                ..Default::default()
            };
            if let Err(e) = p2ptokens_client::run(cfg).await {
                eprintln!("p2ptokens daemon exited: {e:#}");
            }
        });
    });

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running p2ptokens desktop app");
}
