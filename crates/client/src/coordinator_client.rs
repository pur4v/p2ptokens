//! Thin HTTP client for the coordinator tracker API.

use anyhow::{bail, Context, Result};

use p2ptokens_shared::api::{
    MatchManyRequest, MatchManyResponse, MatchResponse, SettleRequest, SettleResponse,
};
use p2ptokens_shared::types::{Heartbeat, LedgerEntry, Match, MatchRequest};

#[derive(Clone)]
pub struct CoordinatorClient {
    base: String,
    http: reqwest::Client,
}

impl CoordinatorClient {
    pub fn new(base: String) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn heartbeat(&self, hb: &Heartbeat) -> Result<()> {
        self.http
            .post(format!("{}/heartbeat", self.base))
            .json(hb)
            .send()
            .await
            .context("heartbeat")?
            .error_for_status()?;
        Ok(())
    }

    pub async fn request_match(&self, req: &MatchRequest) -> Result<MatchResponse> {
        let resp = self
            .http
            .post(format!("{}/match", self.base))
            .json(req)
            .send()
            .await
            .context("match")?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    /// Fan-out matchmaking: ask for up to `count` distinct providers.
    pub async fn request_matches(&self, req: &MatchManyRequest) -> Result<Vec<Match>> {
        let resp = self
            .http
            .post(format!("{}/match_many", self.base))
            .json(req)
            .send()
            .await
            .context("match_many")?
            .error_for_status()?;
        match resp.json::<MatchManyResponse>().await? {
            MatchManyResponse::Matched { matches } => Ok(matches),
            MatchManyResponse::RatioExceeded => {
                bail!("rate limited: serve more to restore your ratio before leeching")
            }
        }
    }

    pub async fn settle(&self, req: &SettleRequest) -> Result<SettleResponse> {
        let resp = self
            .http
            .post(format!("{}/settle", self.base))
            .json(req)
            .send()
            .await
            .context("settle")?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    pub async fn providers(&self) -> Result<Vec<Heartbeat>> {
        let resp = self
            .http
            .get(format!("{}/providers", self.base))
            .send()
            .await
            .context("providers")?
            .error_for_status()?;
        Ok(resp.json().await?)
    }

    /// Fetch a peer's ledger entry. Returns None if the coordinator has no record.
    pub async fn ledger(&self, peer: &str) -> Result<Option<LedgerEntry>> {
        let resp = self
            .http
            .get(format!("{}/ledger/{peer}", self.base))
            .send()
            .await
            .context("ledger")?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        Ok(Some(resp.error_for_status()?.json().await?))
    }
}
