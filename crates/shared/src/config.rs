//! Platform configuration — the single surface an org edits to run its OWN
//! private, branded p2ptokens network.
//!
//! Load precedence (highest wins): CLI flags (applied by each binary) > env vars
//! > `p2ptokens.toml` file > built-in defaults. With no file and no flags the
//! defaults reproduce today's public-network behavior, so existing setups keep
//! working unchanged.

use serde::Deserialize;

/// Isolation + membership for a network.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Namespace that scopes the libp2p protocol id — peers on different `id`s
    /// literally cannot open a completion stream to each other, so an org's
    /// swarm stays separate from the public one.
    pub id: String,
    /// Display name shown in the dashboard.
    pub name: String,
    /// If true, the coordinator requires `join_secret` on every request.
    pub private: bool,
    /// Shared bearer secret gating a private network (empty = open network).
    pub join_secret: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            id: "public".into(),
            name: "p2ptokens".into(),
            private: false,
            join_secret: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CoordinatorConfig {
    /// URL the client uses to reach the coordinator.
    pub url: String,
    /// Address the coordinator binds to.
    pub listen: String,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            url: "https://coordinator.p2ptokens.com".into(),
            listen: "127.0.0.1:4000".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RelayConfig {
    /// Multiaddr of a relay to reserve a slot on (for reachability behind NAT).
    pub addr: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ClientConfig {
    /// Local HTTP address (dashboard + `/v1` endpoint).
    pub http: String,
    /// libp2p listen multiaddr.
    pub p2p_listen: String,
    /// Max concurrent jobs to serve.
    pub capacity: u32,
    /// Run as a public circuit-relay server (rendezvous for NAT'd peers).
    pub relay: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            http: "127.0.0.1:8080".into(),
            p2p_listen: "/ip4/0.0.0.0/tcp/0".into(),
            capacity: 4,
            relay: false,
        }
    }
}

/// White-label branding surfaced to the dashboard via `/api/config`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct BrandConfig {
    pub product_name: String,
    pub tagline: String,
    pub accent: String,
    pub amber: String,
    pub website: String,
    pub github: String,
    pub support_email: String,
    /// Optional logo image URL; when set the dashboard shows it instead of the
    /// ASCII banner.
    pub logo_url: String,
}

impl Default for BrandConfig {
    fn default() -> Self {
        Self {
            product_name: "p2ptokens".into(),
            tagline: "distributed inference swarm — seed to leech".into(),
            accent: "#33ff88".into(),
            amber: "#ffb000".into(),
            website: "https://p2ptokens.com".into(),
            github: "https://github.com/pur4v/p2ptokens".into(),
            support_email: String::new(),
            logo_url: String::new(),
        }
    }
}

/// The whole platform config. A missing section falls back to that section's
/// defaults; a missing key falls back to that struct's `Default`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlatformConfig {
    pub network: NetworkConfig,
    pub coordinator: CoordinatorConfig,
    pub relay: RelayConfig,
    pub client: ClientConfig,
    pub brand: BrandConfig,
}

impl PlatformConfig {
    /// Load config: explicit `path`, else `$P2PTOKENS_CONFIG`, else
    /// `./p2ptokens.toml` if present, else built-in defaults; then apply env
    /// overrides. CLI flags are layered on top by each binary after this.
    pub fn load(path: Option<&str>) -> Self {
        let mut cfg = Self::from_file(path).unwrap_or_default();
        cfg.apply_env();
        cfg
    }

    fn from_file(path: Option<&str>) -> Option<Self> {
        let file = path
            .map(str::to_string)
            .or_else(|| std::env::var("P2PTOKENS_CONFIG").ok())
            .or_else(|| {
                std::path::Path::new("p2ptokens.toml")
                    .exists()
                    .then(|| "p2ptokens.toml".to_string())
            })?;
        match std::fs::read_to_string(&file) {
            Ok(text) => match toml::from_str::<Self>(&text) {
                Ok(cfg) => Some(cfg),
                Err(e) => {
                    eprintln!("p2ptokens: ignoring malformed {file}: {e}");
                    None
                }
            },
            Err(e) => {
                eprintln!("p2ptokens: cannot read {file}: {e}");
                None
            }
        }
    }

    /// Back-compat env overrides (these worked before the config file existed).
    fn apply_env(&mut self) {
        if let Ok(v) = std::env::var("P2PTOKENS_COORDINATOR") {
            self.coordinator.url = v;
        }
        if let Ok(v) = std::env::var("P2PTOKENS_NETWORK_ID") {
            self.network.id = v;
        }
        if let Ok(v) = std::env::var("P2PTOKENS_JOIN_SECRET") {
            self.network.join_secret = v;
            self.network.private = true;
        }
    }

    /// The join secret, if this is a private network (else None).
    pub fn join_secret(&self) -> Option<String> {
        let s = self.network.join_secret.trim();
        (self.network.private && !s.is_empty()).then(|| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_reproduce_public_network() {
        let c = PlatformConfig::default();
        assert_eq!(c.network.id, "public");
        assert_eq!(c.brand.product_name, "p2ptokens");
        assert!(c.join_secret().is_none());
    }

    #[test]
    fn partial_toml_fills_missing_keys_from_defaults() {
        let c: PlatformConfig = toml::from_str(
            r#"
            [network]
            id = "acme"
            private = true
            join_secret = "s3cret"
            [brand]
            product_name = "Acme AI"
        "#,
        )
        .unwrap();
        assert_eq!(c.network.id, "acme");
        assert_eq!(c.network.name, "p2ptokens"); // missing key -> default
        assert_eq!(c.brand.product_name, "Acme AI");
        assert_eq!(c.brand.accent, "#33ff88"); // missing key -> default
        assert_eq!(c.coordinator.listen, "127.0.0.1:4000"); // missing section -> default
        assert_eq!(c.join_secret().as_deref(), Some("s3cret"));
    }
}
