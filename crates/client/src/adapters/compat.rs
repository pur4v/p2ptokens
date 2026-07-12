//! Generic chat-completions backend adapter. Speaks the widely-adopted
//! `/chat/completions` streaming wire format, so it works against ANY compatible
//! endpoint URL — a local server (vLLM, llama.cpp, LM Studio, Ollama's compat
//! port) or a hosted gateway. BYO-credentials; the user supplies the URL and
//! (optionally) an API key and bears their own ToS risk (design Q13).

use anyhow::{Context, Result};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use super::{estimate_tokens, AdapterRequest};
use p2ptokens_shared::types::{CompletionDelta, ModelId};

pub struct CompatAdapter {
    base_url: String,
    api_key: Option<String>,
    models: Vec<String>,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct StreamChunk {
    #[serde(default)]
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}
#[derive(Deserialize)]
struct Choice {
    #[serde(default)]
    delta: Delta,
    #[serde(default)]
    finish_reason: Option<String>,
}
#[derive(Deserialize, Default)]
struct Delta {
    #[serde(default)]
    content: Option<String>,
}
#[derive(Deserialize)]
struct Usage {
    #[serde(default)]
    completion_tokens: Option<u64>,
}

impl CompatAdapter {
    pub fn new(base_url: String, api_key: Option<String>, models: Vec<String>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
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
        let body = json!({
            "model": req.model,
            "messages": req.messages,
            "stream": true,
            "stream_options": { "include_usage": true },
            "temperature": req.params.temperature,
            "top_p": req.params.top_p,
            "max_tokens": req.params.max_tokens,
            "stop": req.params.stop,
        });
        let mut request = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .json(&body);
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        let resp = request
            .send()
            .await
            .context("compat /chat/completions")?
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
                let data = data.trim();
                if data == "[DONE]" {
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
                let c: StreamChunk = match serde_json::from_str(data) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Some(u) = c.usage.and_then(|u| u.completion_tokens) {
                    cumulative = u;
                }
                if let Some(choice) = c.choices.into_iter().next() {
                    if let Some(fr) = choice.finish_reason {
                        finish = fr;
                    }
                    if let Some(text) = choice.delta.content {
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
            }
        }
        Ok(())
    }
}
