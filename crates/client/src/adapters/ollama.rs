//! Ollama backend adapter — talks to a local Ollama daemon's REST API.

use anyhow::{Context, Result};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use super::files;
use super::{estimate_tokens, AdapterRequest};
use p2ptokens_shared::types::{CompletionDelta, MessageContent};
use p2ptokens_shared::types::{ContentPart, ModelId};

pub struct OllamaAdapter {
    base_url: String,
    http: reqwest::Client,
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}
#[derive(Deserialize)]
struct TagModel {
    name: String,
    #[serde(default)]
    details: Option<TagDetails>,
}
#[derive(Deserialize)]
struct TagDetails {
    #[serde(default)]
    quantization_level: Option<String>,
}

#[derive(Deserialize)]
struct ChatChunk {
    #[serde(default)]
    message: Option<ChatMsg>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    eval_count: Option<u64>,
}
#[derive(Deserialize)]
struct ChatMsg {
    #[serde(default)]
    content: String,
}

impl OllamaAdapter {
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url
            .unwrap_or_else(|| "http://127.0.0.1:11434".to_string())
            .trim_end_matches('/')
            .to_string();
        Self {
            base_url,
            http: reqwest::Client::new(),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<ModelId>> {
        let resp = self
            .http
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .context("ollama /api/tags")?
            .error_for_status()?;
        let body: TagsResponse = resp.json().await?;
        Ok(body
            .models
            .into_iter()
            .map(|m| ModelId {
                name: m.name,
                quant: m.details.and_then(|d| d.quantization_level),
            })
            .collect())
    }

    pub async fn stream(
        &self,
        req: AdapterRequest,
        tx: mpsc::Sender<CompletionDelta>,
    ) -> Result<()> {
        let messages: Vec<serde_json::Value> = req.messages.iter().map(ollama_message).collect();
        let body = json!({
            "model": req.model,
            "messages": messages,
            "stream": true,
            "options": {
                "temperature": req.params.temperature,
                "top_p": req.params.top_p,
                "num_predict": req.params.max_tokens,
                "stop": req.params.stop,
            }
        });
        let resp = self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .context("ollama /api/chat")?
            .error_for_status()?;

        let mut stream = resp.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        let mut estimated: u64 = 0;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buf.extend_from_slice(&chunk);
            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buf.drain(..=pos).collect();
                let s = String::from_utf8_lossy(&line[..line.len() - 1])
                    .trim()
                    .to_string();
                if s.is_empty() {
                    continue;
                }
                let c: ChatChunk = serde_json::from_str(&s)?;
                let text = c.message.map(|m| m.content).unwrap_or_default();
                if !text.is_empty() {
                    estimated += estimate_tokens(&text);
                }
                let cumulative = c.eval_count.unwrap_or(estimated);
                if !text.is_empty()
                    && tx
                        .send(CompletionDelta {
                            text,
                            cumulative_tokens: cumulative,
                            done: false,
                            finish_reason: None,
                        })
                        .await
                        .is_err()
                {
                    return Ok(()); // consumer went away
                }
                if c.done {
                    let _ = tx
                        .send(CompletionDelta {
                            text: String::new(),
                            cumulative_tokens: cumulative,
                            done: true,
                            finish_reason: Some(c.done_reason.unwrap_or_else(|| "stop".into())),
                        })
                        .await;
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

/// Translate one message into Ollama's `/api/chat` shape: text (plus any extracted
/// file text) as `content`, and images as bare base64 in `images` (its vision API).
fn ollama_message(m: &p2ptokens_shared::types::ChatMessage) -> serde_json::Value {
    match &m.content {
        MessageContent::Text(s) => json!({ "role": m.role, "content": s }),
        MessageContent::Parts(parts) => {
            let mut text = String::new();
            let mut images: Vec<String> = Vec::new();
            for p in parts {
                match p {
                    ContentPart::Text { text: t } => {
                        if !text.is_empty() {
                            text.push('\n');
                        }
                        text.push_str(t);
                    }
                    ContentPart::ImageUrl { image_url } => {
                        // Ollama wants bare base64 (no data: prefix); pass remote URLs through.
                        let (_, payload) = files::parse_data_uri(&image_url.url);
                        images.push(payload);
                    }
                    ContentPart::File { file } => {
                        if !text.is_empty() {
                            text.push('\n');
                        }
                        text.push_str(&files::as_prompt_block(file));
                    }
                }
            }
            if images.is_empty() {
                json!({ "role": m.role, "content": text })
            } else {
                json!({ "role": m.role, "content": text, "images": images })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use p2ptokens_shared::types::{ChatMessage, ImageUrl};

    #[test]
    fn image_part_becomes_bare_base64_in_images() {
        let m = ChatMessage {
            role: "user".into(),
            content: MessageContent::Parts(vec![
                ContentPart::Text { text: "hi".into() },
                ContentPart::ImageUrl {
                    image_url: ImageUrl {
                        url: "data:image/png;base64,QUJD".into(),
                    },
                },
            ]),
        };
        let v = ollama_message(&m);
        assert_eq!(v["content"], "hi");
        assert_eq!(v["images"][0], "QUJD"); // data: prefix stripped
    }

    #[test]
    fn plain_text_has_no_images_field() {
        let m = ChatMessage::text("user", "hello");
        let v = ollama_message(&m);
        assert_eq!(v["content"], "hello");
        assert!(v.get("images").is_none());
    }
}
