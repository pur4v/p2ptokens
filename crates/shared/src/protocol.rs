//! The peer-to-peer completion protocol spoken over a libp2p stream between a
//! leecher (consumer) and a seeder (provider). Messages are length-prefixed JSON
//! (u32 big-endian length, then a JSON body).

use anyhow::Result;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use serde::{Deserialize, Serialize};

use crate::receipts::SignedReceipt;
use crate::types::{ChatCompletionParams, ChatMessage};

/// Protocol id negotiated on the libp2p stream.
pub const COMPLETION_PROTOCOL: &str = "/p2ptokens/completion/1.0.0";

const MAX_FRAME: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Wire {
    /// consumer -> provider: opens the job
    Request {
        job_id: String,
        model: String,
        /// consumer's public key (b64 protobuf) so the provider can verify receipts
        consumer_pubkey: String,
        messages: Vec<ChatMessage>,
        params: ChatCompletionParams,
    },
    /// provider -> consumer: a streamed chunk of output
    Chunk {
        seq: u64,
        text: String,
        cumulative_tokens: u64,
    },
    /// provider -> consumer: generation finished
    Done {
        finish_reason: String,
        cumulative_tokens: u64,
    },
    /// provider -> consumer: fatal error serving the job
    Error { message: String },
    /// consumer -> provider: signed acknowledgement of cumulative tokens received
    Receipt { receipt: SignedReceipt },
}

pub async fn write_msg<W: AsyncWrite + Unpin>(w: &mut W, msg: &Wire) -> Result<()> {
    let bytes = serde_json::to_vec(msg)?;
    let len = (bytes.len() as u32).to_be_bytes();
    w.write_all(&len).await?;
    w.write_all(&bytes).await?;
    w.flush().await?;
    Ok(())
}

/// Read one framed message. Returns `Ok(None)` on a clean end-of-stream.
pub async fn read_msg<R: AsyncRead + Unpin>(r: &mut R) -> Result<Option<Wire>> {
    let mut len = [0u8; 4];
    match r.read_exact(&mut len).await {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e.into()),
    }
    let n = u32::from_be_bytes(len) as usize;
    if n > MAX_FRAME {
        anyhow::bail!("frame too large: {n} bytes");
    }
    let mut buf = vec![0u8; n];
    r.read_exact(&mut buf).await?;
    Ok(Some(serde_json::from_slice(&buf)?))
}
