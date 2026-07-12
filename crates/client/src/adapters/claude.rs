//! Claude backend adapter — proxies the caller's own Anthropic API key
//! (BYO-credentials). Same user-borne ToS risk as any hosted provider (design Q13).

use anyhow::{Context, Result};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use super::{estimate_tokens, split_system, AdapterRequest};
use p2ptokens_shared::types::{ChatMessage, CompletionDelta, ModelId};

pub struct ClaudeAdapter {
    api_key: String,
    base_url: String,
    models: Vec<String>,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct Event {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    delta: Option<Delta>,
    #[serde(default)]
    usage: Option<Usage>,
}
#[derive(Deserialize, Default)]
struct Delta {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    stop_reason: Option<String>,
}
#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    output_tokens: Option<u64>,
}

impl ClaudeAdapter {
    pub fn new(api_key: String, models: Vec<String>, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url
                .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string())
                .trim_end_matches('/')
                .to_string(),
            models,
            http: reqwest::Client::new(),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<ModelId>> {
        Ok(self
            .models
            .iter()
            .map(|m| ModelId {
                name: m.clone(),
                quant: None,
            })
            .collect())
    }

    pub async fn stream(
        &self,
        req: AdapterRequest,
        tx: mpsc::Sender<CompletionDelta>,
    ) -> Result<()> {
        let (system, rest) = split_system(&req.messages);
        let msgs: Vec<ChatMessage> = rest.into_iter().cloned().collect();
        let body = json!({
            "model": req.model,
            "system": system,
            "messages": msgs,
            "stream": true,
            "max_tokens": req.params.max_tokens.unwrap_or(1024),
            "temperature": req.params.temperature,
            "top_p": req.params.top_p,
            "stop_sequences": req.params.stop,
        });
        let resp = self
            .http
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .context("anthropic /messages")?
            .error_for_status()?;

        let mut stream = resp.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        let mut estimated: u64 = 0;
        let mut cumulative: u64 = 0;
        let mut finish = "stop".to_string();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let s = String::from_utf8_lossy(&line[..line.len() - 1])
                    .trim()
                    .to_string();
                let Some(data) = s.strip_prefix("data:") else {
                    continue;
                };
                let ev: Event = match serde_json::from_str(data.trim()) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                match ev.kind.as_str() {
                    "content_block_delta" => {
                        if let Some(text) = ev.delta.and_then(|d| d.text) {
                            if !text.is_empty() {
                                estimated += estimate_tokens(&text);
                                if cumulative < estimated {
                                    cumulative = estimated;
                                }
                                if tx
                                    .send(CompletionDelta {
                                        text,
                                        cumulative_tokens: cumulative,
                                        done: false,
                                        finish_reason: None,
                                    })
                                    .await
                                    .is_err()
                                {
                                    return Ok(());
                                }
                            }
                        }
                    }
                    "message_delta" => {
                        if let Some(u) = ev.usage.and_then(|u| u.output_tokens) {
                            cumulative = u;
                        }
                        if let Some(d) = ev.delta {
                            if let Some(sr) = d.stop_reason {
                                finish = sr;
                            }
                        }
                    }
                    "message_stop" => {
                        let _ = tx
                            .send(CompletionDelta {
                                text: String::new(),
                                cumulative_tokens: cumulative,
                                done: true,
                                finish_reason: Some(finish.clone()),
                            })
                            .await;
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
