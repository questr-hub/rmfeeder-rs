use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};

use crate::expand_tilde_path;

pub struct StateDb {
    conn: Connection,
    seen_in_run: HashSet<String>,
}

impl StateDb {
    pub fn should_emit(&mut self, key: &str) -> rusqlite::Result<bool> {
        if self.seen_in_run.contains(key) {
            return Ok(false);
        }

        let exists = self
            .conn
            .query_row("SELECT 1 FROM seen WHERE url = ?1 LIMIT 1", [key], |_| {
                Ok(())
            })
            .optional()?
            .is_some();
        Ok(!exists)
    }

    pub fn mark_seen(&mut self, key: &str) -> rusqlite::Result<()> {
        if self.seen_in_run.contains(key) {
            return Ok(());
        }
        let now = chrono::Utc::now().timestamp();
        self.conn.execute(
            "INSERT OR IGNORE INTO seen (url, seen_at) VALUES (?1, ?2)",
            params![key, now],
        )?;
        self.seen_in_run.insert(key.to_string());
        Ok(())
    }
}

pub fn init_state_db(
    clear_state: bool,
    custom_path: Option<String>,
) -> Result<StateDb, Box<dyn std::error::Error>> {
    let path = match custom_path {
        Some(path) => expand_tilde_path(&path),
        None => default_state_path()?,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS seen (url TEXT PRIMARY KEY, seen_at INTEGER NOT NULL)",
        [],
    )?;

    if clear_state {
        conn.execute("DELETE FROM seen", [])?;
    }

    Ok(StateDb {
        conn,
        seen_in_run: HashSet::new(),
    })
}

pub fn default_state_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    Ok(Path::new(&home)
        .join(".local")
        .join("share")
        .join("rmfeeder")
        .join("rmfeeder_state.sqlite"))
}
