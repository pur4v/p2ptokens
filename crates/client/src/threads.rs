//! Local chat history: a durable SQLite store (via `rusqlite`, feature `bundled`,
//! so the engine is compiled into the binary — no system dependency). One DB file
//! lives in the client data-dir next to the identity key. Models the thread /
//! message shape the chat UI expects (inspired by synapse's chat schema), kept
//! deliberately local and simple — no Postgres, no server-side accounts.

use std::path::Path;
use std::sync::Mutex;

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use p2ptokens_shared::types::{ChatMessage, MessageContent};

/// A conversation thread (summary row for the history sidebar).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThreadRow {
    pub id: String,
    pub title: String,
    pub created: i64,
    pub updated: i64,
}

/// One stored message. `content` is the raw `MessageContent` JSON (string or
/// multimodal parts), so history round-trips images/files, not just text.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageRow {
    pub id: String,
    pub role: String,
    pub content: serde_json::Value,
    pub created: i64,
}

pub struct ThreadStore {
    conn: Mutex<Connection>,
}

impl ThreadStore {
    /// Open (creating if needed) the history DB at `path` and ensure the schema.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("open history db")?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS threads (
                 id      TEXT PRIMARY KEY,
                 title   TEXT NOT NULL DEFAULT '',
                 created INTEGER NOT NULL,
                 updated INTEGER NOT NULL
             );
             CREATE TABLE IF NOT EXISTS messages (
                 id        TEXT PRIMARY KEY,
                 thread_id TEXT NOT NULL,
                 role      TEXT NOT NULL,
                 content   TEXT NOT NULL,
                 created   INTEGER NOT NULL,
                 seq       INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id, seq);",
        )
        .context("init history schema")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Create a new thread row.
    pub fn create_thread(&self, id: &str, title: &str, now: i64) -> Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute(
            "INSERT OR IGNORE INTO threads (id, title, created, updated) VALUES (?1, ?2, ?3, ?3)",
            params![id, title, now],
        )?;
        Ok(())
    }

    /// Append a message to a thread and bump the thread's `updated` time.
    pub fn add_message(
        &self,
        msg_id: &str,
        thread_id: &str,
        role: &str,
        content_json: &str,
        now: i64,
    ) -> Result<()> {
        let c = self.conn.lock().unwrap();
        let seq: i64 = c
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM messages WHERE thread_id = ?1",
                params![thread_id],
                |r| r.get(0),
            )
            .unwrap_or(1);
        c.execute(
            "INSERT INTO messages (id, thread_id, role, content, created, seq) VALUES (?1,?2,?3,?4,?5,?6)",
            params![msg_id, thread_id, role, content_json, now, seq],
        )?;
        c.execute(
            "UPDATE threads SET updated = ?2 WHERE id = ?1",
            params![thread_id, now],
        )?;
        Ok(())
    }

    /// List threads newest-first, optionally filtered by a title substring.
    pub fn list_threads(&self, query: Option<&str>) -> Result<Vec<ThreadRow>> {
        let c = self.conn.lock().unwrap();
        let like = format!("%{}%", query.unwrap_or("").replace('%', ""));
        let mut stmt = c.prepare(
            "SELECT id, title, created, updated FROM threads
             WHERE (?1 = '%%' OR title LIKE ?1) ORDER BY updated DESC LIMIT 200",
        )?;
        let rows = stmt
            .query_map(params![like], |r| {
                Ok(ThreadRow {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    created: r.get(2)?,
                    updated: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// All messages in a thread, oldest-first.
    pub fn get_messages(&self, thread_id: &str) -> Result<Vec<MessageRow>> {
        let c = self.conn.lock().unwrap();
        let mut stmt = c.prepare(
            "SELECT id, role, content, created FROM messages WHERE thread_id = ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt
            .query_map(params![thread_id], |r| {
                let content: String = r.get(2)?;
                Ok(MessageRow {
                    id: r.get(0)?,
                    role: r.get(1)?,
                    content: serde_json::from_str(&content)
                        .unwrap_or(serde_json::Value::String(content)),
                    created: r.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// Prior turns of a thread as `ChatMessage`s, for building request context.
    pub fn history_as_chat(&self, thread_id: &str) -> Result<Vec<ChatMessage>> {
        let msgs = self.get_messages(thread_id)?;
        Ok(msgs
            .into_iter()
            .filter(|m| m.role == "user" || m.role == "assistant")
            .map(|m| ChatMessage {
                role: m.role,
                content: serde_json::from_value::<MessageContent>(m.content.clone())
                    .unwrap_or_else(|_| MessageContent::Text(value_to_text(&m.content))),
            })
            .collect())
    }

    pub fn delete_thread(&self, id: &str) -> Result<()> {
        let c = self.conn.lock().unwrap();
        c.execute("DELETE FROM messages WHERE thread_id = ?1", params![id])?;
        c.execute("DELETE FROM threads WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn value_to_text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_and_message_roundtrip() {
        let dir = std::env::temp_dir().join(format!("p2p-hist-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&dir);
        let s = ThreadStore::open(&dir).unwrap();
        s.create_thread("t1", "hello world", 1000).unwrap();
        s.add_message("m1", "t1", "user", "\"hi there\"", 1001)
            .unwrap();
        s.add_message("m2", "t1", "assistant", "\"hello!\"", 1002)
            .unwrap();

        let threads = s.list_threads(None).unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].title, "hello world");

        let msgs = s.get_messages("t1").unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "user");

        let chat = s.history_as_chat("t1").unwrap();
        assert_eq!(chat.len(), 2);
        assert_eq!(chat[0].content.to_text(), "hi there");

        // search + delete
        assert_eq!(s.list_threads(Some("world")).unwrap().len(), 1);
        assert_eq!(s.list_threads(Some("nope")).unwrap().len(), 0);
        s.delete_thread("t1").unwrap();
        assert_eq!(s.list_threads(None).unwrap().len(), 0);
        assert_eq!(s.get_messages("t1").unwrap().len(), 0);
        let _ = std::fs::remove_file(&dir);
    }
}
