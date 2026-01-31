use anyhow::{Context, Result};
use rusqlite::Connection;

/// Initialize the memory module database schema.
/// Creates the memory table, FTS5 virtual table, and triggers for auto-syncing.
pub fn init_memory_schema(conn: &Connection) -> Result<()> {
    // Create memory table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS memory (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            category TEXT,
            importance INTEGER DEFAULT 5,
            created_at INTEGER
        )
        "#,
        [],
    )
    .context("Failed to create memory table")?;

    // Create FTS5 virtual table for full-text search
    conn.execute(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
            content,
            content='memory',
            content_rowid='id'
        )
        "#,
        [],
    )
    .context("Failed to create memory_fts virtual table")?;

    // Create trigger for auto-syncing FTS index on insert
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS memory_ai AFTER INSERT ON memory BEGIN
            INSERT INTO memory_fts(rowid, content) VALUES (new.id, new.content);
        END
        "#,
        [],
    )
    .context("Failed to create memory_ai trigger")?;

    // Create trigger for auto-syncing FTS index on delete
    conn.execute(
        r#"
        CREATE TRIGGER IF NOT EXISTS memory_ad AFTER DELETE ON memory BEGIN
            INSERT INTO memory_fts(memory_fts, rowid, content) VALUES('delete', old.id, old.content);
        END
        "#,
        [],
    )
    .context("Failed to create memory_ad trigger")?;

    // Rebuild FTS index from content table (syncs pre-existing rows not covered by triggers)
    conn.execute("INSERT INTO memory_fts(memory_fts) VALUES('rebuild')", [])
        .context("Failed to rebuild memory_fts index")?;

    Ok(())
}

