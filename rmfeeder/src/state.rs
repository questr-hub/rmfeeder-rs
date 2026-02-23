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

#[cfg(test)]
mod tests {
    use super::init_state_db;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("rmfeeder-state-{name}-{nanos}.sqlite"))
    }

    #[test]
    fn state_marks_and_skips_seen_keys() {
        let path = temp_db_path("mark-skip");
        let path_str = path.to_string_lossy().to_string();

        let mut state = init_state_db(false, Some(path_str.clone())).expect("init state");
        assert!(state.should_emit("https://example.com/a").expect("query state"));
        state.mark_seen("https://example.com/a").expect("mark seen");
        assert!(!state.should_emit("https://example.com/a").expect("query state"));

        let mut reopened = init_state_db(false, Some(path_str)).expect("reopen state");
        assert!(
            !reopened
                .should_emit("https://example.com/a")
                .expect("query persisted state")
        );

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn clear_state_removes_existing_seen_entries() {
        let path = temp_db_path("clear");
        let path_str = path.to_string_lossy().to_string();

        let mut state = init_state_db(false, Some(path_str.clone())).expect("init state");
        state.mark_seen("yt::https://youtube.com/watch?v=abc")
            .expect("mark seen");
        drop(state);

        let mut cleared = init_state_db(true, Some(path_str)).expect("clear state");
        assert!(
            cleared
                .should_emit("yt::https://youtube.com/watch?v=abc")
                .expect("query cleared state")
        );

        std::fs::remove_file(path).ok();
    }
}
