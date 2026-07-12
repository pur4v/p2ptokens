//! Client configuration and on-disk identity persistence.

use anyhow::{Context, Result};
use std::path::Path;

use p2ptokens_shared::crypto;

/// Which backends this peer will serve from (BYO-credentials, Q13).
#[derive(Debug, Clone, Default)]
pub struct BackendConfig {
    pub ollama: bool,
    pub ollama_url: Option<String>,
    /// a generic chat-completions endpoint at any URL
    pub endpoint_url: Option<String>,
    pub endpoint_key: Option<String>,
    pub endpoint_models: Vec<String>,
    pub claude_key: Option<String>,
    pub claude_models: Vec<String>,
}

impl BackendConfig {
    /// Read backend configuration from environment variables.
    pub fn from_env() -> Self {
        let csv = |k: &str| -> Vec<String> {
            std::env::var(k)
                .ok()
                .map(|s| {
                    s.split(',')
                        .map(|x| x.trim().to_string())
                        .filter(|x| !x.is_empty())
                        .collect()
                })
                .unwrap_or_default()
        };
        // Ollama on by default unless explicitly disabled.
        let ollama = std::env::var("P2P_OLLAMA")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .unwrap_or(true);
        Self {
            ollama,
            ollama_url: std::env::var("P2P_OLLAMA_URL").ok(),
            endpoint_url: std::env::var("P2P_ENDPOINT_URL").ok(),
            endpoint_key: std::env::var("P2P_ENDPOINT_KEY").ok(),
            endpoint_models: csv("P2P_ENDPOINT_MODELS"),
            claude_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            claude_models: csv("P2P_CLAUDE_MODELS"),
        }
    }
}

/// Load the persisted ed25519 identity, or create and persist a new one.
pub fn load_or_create_identity(dir: &Path) -> Result<crypto::Identity> {
    std::fs::create_dir_all(dir).context("create data dir")?;
    let path = dir.join("identity.key");
    if path.exists() {
        let s = std::fs::read_to_string(&path).context("read identity")?;
        crypto::import_secret(&s)
    } else {
        let kp = crypto::generate_identity();
        std::fs::write(&path, crypto::export_secret(&kp)?).context("write identity")?;
        Ok(kp)
    }
}
