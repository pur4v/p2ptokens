//! Backend adapters normalize every inference source to the same streamed
//! chat-completion shape. v1 backends: Ollama (local), a generic compatible HTTP
//! endpoint (any URL), and Claude — matched purely by model name
//! (source-agnostic, Q5/Q13). Enum dispatch keeps it simple (no async-trait).

mod claude;
mod compat;
pub mod files;
mod ollama;

pub use claude::ClaudeAdapter;
pub use compat::CompatAdapter;
pub use ollama::OllamaAdapter;

use anyhow::Result;
use tokio::sync::mpsc;

use p2ptokens_shared::types::{ChatCompletionParams, ChatMessage, CompletionDelta, ModelId};

pub struct AdapterRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub params: ChatCompletionParams,
}

pub enum Adapter {
    Ollama(OllamaAdapter),
    Compat(CompatAdapter),
    Claude(ClaudeAdapter),
}

impl Adapter {
    pub fn backend(&self) -> &'static str {
        match self {
            Adapter::Ollama(_) => "ollama",
            Adapter::Compat(_) => "endpoint",
            Adapter::Claude(_) => "claude",
        }
    }

    pub async fn list_models(&self) -> Result<Vec<ModelId>> {
        match self {
            Adapter::Ollama(a) => a.list_models().await,
            Adapter::Compat(a) => a.list_models().await,
            Adapter::Claude(a) => a.list_models().await,
        }
    }

    pub async fn stream(
        &self,
        req: AdapterRequest,
        tx: mpsc::Sender<CompletionDelta>,
    ) -> Result<()> {
        match self {
            Adapter::Ollama(a) => a.stream(req, tx).await,
            Adapter::Compat(a) => a.stream(req, tx).await,
            Adapter::Claude(a) => a.stream(req, tx).await,
        }
    }
}

/// Rough token estimate when a backend does not report exact counts (~4 chars/token).
pub fn estimate_tokens(text: &str) -> u64 {
    ((text.chars().count() as u64) / 4).max(1)
}

/// Split chat messages: Claude wants `system` separate from the turn list.
pub fn split_system(messages: &[ChatMessage]) -> (String, Vec<&ChatMessage>) {
    let mut system = String::new();
    let mut rest = Vec::new();
    for m in messages {
        if m.role == "system" {
            if !system.is_empty() {
                system.push('\n');
            }
            system.push_str(&m.content.to_text());
        } else {
            rest.push(m);
        }
    }
    (system, rest)
}
