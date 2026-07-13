//! Claude backend adapter — proxies the caller's own Anthropic API key
//! (BYO-credentials). Same user-borne ToS risk as any hosted provider (design Q13).

use anyhow::{Context, Result};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use super::files;
use super::{estimate_tokens, split_system, AdapterRequest};
use p2ptokens_shared::types::{ChatMessage, CompletionDelta, ContentPart, MessageContent, ModelId};

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
        let msgs: Vec<serde_json::Value> = rest.into_iter().map(claude_message).collect();
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

/// Translate one message into the Anthropic Messages shape. Plain text stays a
/// string; multimodal messages become content blocks (text / image / document).
fn claude_message(m: &ChatMessage) -> serde_json::Value {
    match &m.content {
        MessageContent::Text(s) => json!({ "role": m.role, "content": s }),
        MessageContent::Parts(parts) => {
            let mut blocks: Vec<serde_json::Value> = Vec::new();
            for p in parts {
                match p {
                    ContentPart::Text { text } => {
                        blocks.push(json!({ "type": "text", "text": text }));
                    }
                    ContentPart::ImageUrl { image_url } => {
                        let (mime, data) = files::parse_data_uri(&image_url.url);
                        if data.starts_with("http") && mime.is_empty() {
                            blocks.push(json!({
                                "type": "image",
                                "source": { "type": "url", "url": image_url.url },
                            }));
                        } else {
                            let media_type = if mime.is_empty() {
                                "image/png".into()
                            } else {
                                mime
                            };
                            blocks.push(json!({
                                "type": "image",
                                "source": { "type": "base64", "media_type": media_type, "data": data },
                            }));
                        }
                    }
                    ContentPart::File { file } => {
                        if files::is_pdf(file) {
                            // Native PDF document block (Anthropic supports this directly).
                            let (_, data) = files::parse_data_uri(&file.data);
                            blocks.push(json!({
                                "type": "document",
                                "source": {
                                    "type": "base64",
                                    "media_type": "application/pdf",
                                    "data": data,
                                },
                            }));
                        } else {
                            blocks.push(
                                json!({ "type": "text", "text": files::as_prompt_block(file) }),
                            );
                        }
                    }
                }
            }
            json!({ "role": m.role, "content": blocks })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p2ptokens_shared::types::{FileData, ImageUrl};

    #[test]
    fn image_becomes_base64_source_block() {
        let m = ChatMessage {
            role: "user".into(),
            content: MessageContent::Parts(vec![ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: "data:image/jpeg;base64,QUJD".into(),
                },
            }]),
        };
        let v = claude_message(&m);
        let block = &v["content"][0];
        assert_eq!(block["type"], "image");
        assert_eq!(block["source"]["type"], "base64");
        assert_eq!(block["source"]["media_type"], "image/jpeg");
        assert_eq!(block["source"]["data"], "QUJD");
    }

    #[test]
    fn pdf_becomes_native_document_block() {
        let m = ChatMessage {
            role: "user".into(),
            content: MessageContent::Parts(vec![ContentPart::File {
                file: FileData {
                    filename: "a.pdf".into(),
                    mime: "application/pdf".into(),
                    data: "data:application/pdf;base64,QUJD".into(),
                },
            }]),
        };
        let v = claude_message(&m);
        let block = &v["content"][0];
        assert_eq!(block["type"], "document");
        assert_eq!(block["source"]["media_type"], "application/pdf");
        assert_eq!(block["source"]["data"], "QUJD");
    }
}
