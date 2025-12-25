#![allow(dead_code)]

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::models::{EventType, KeystrokeEvent};

const KEY_FILE_NAME: &str = ".key";
const KEY_LENGTH: usize = 32;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref();
        let is_memory = db_path.to_str() == Some(":memory:");

        let conn = Connection::open(db_path)?;

        if !is_memory {
            let key = Self::get_or_create_key(db_path)?;
            Self::apply_encryption(&conn, &key)?;
        }

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", -20000)?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        conn.pragma_update(None, "mmap_size", 268435456)?;
        conn.pragma_update(None, "page_size", 4096)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        let mut db = Self { conn };
        db.initialize_schema()?;

        Ok(db)
    }

    fn get_or_create_key(db_path: &Path) -> Result<String> {
        let key_path = Self::key_path(db_path)?;

        if key_path.exists() {
            let mut key = String::new();
            File::open(&key_path)
                .context("Failed to open key file")?
                .read_to_string(&mut key)
                .context("Failed to read key file")?;
            return Ok(key.trim().to_string());
        }

        let key = Self::generate_random_key();

        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(&key_path).context("Failed to create key file")?;
        file.write_all(key.as_bytes())?;

        let mut perms = fs::metadata(&key_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&key_path, perms)?;

        Ok(key)
    }

    fn key_path(db_path: &Path) -> Result<PathBuf> {
        let parent = db_path
            .parent()
            .context("Database path has no parent directory")?;
        Ok(parent.join(KEY_FILE_NAME))
    }

    fn generate_random_key() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let mut state = seed;
        let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

        (0..KEY_LENGTH)
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                let idx = ((state >> 33) as usize) % charset.len();
                charset[idx] as char
            })
            .collect()
    }

    fn apply_encryption(conn: &Connection, key: &str) -> Result<()> {
        conn.pragma_update(None, "key", key)?;
        Ok(())
    }

    fn initialize_schema(&mut self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS keystroke_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                key_code INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                modifiers TEXT,
                application TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_timestamp 
                ON keystroke_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_key_code 
                ON keystroke_events(key_code);
            CREATE INDEX IF NOT EXISTS idx_application 
                ON keystroke_events(application);
            CREATE INDEX IF NOT EXISTS idx_timestamp_key 
                ON keystroke_events(timestamp, key_code);

            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
            );

            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now') * 1000)
            );
            "#,
        )?;

        Ok(())
    }

    pub fn insert_event(&self, event: &KeystrokeEvent) -> Result<()> {
        let modifiers_json = serde_json::to_string(&event.modifiers)?;

        self.conn.execute(
            "INSERT INTO keystroke_events (timestamp, key_code, event_type, modifiers, application)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                event.timestamp,
                event.key_code,
                event.event_type.as_str(),
                modifiers_json,
                event.application,
            ],
        )?;

        Ok(())
    }

    pub fn insert_events_batch(&mut self, events: &[KeystrokeEvent]) -> Result<()> {
        let tx = self.conn.transaction()?;

        for event in events {
            let modifiers_json = serde_json::to_string(&event.modifiers)?;

            tx.execute(
                "INSERT INTO keystroke_events (timestamp, key_code, event_type, modifiers, application)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    event.timestamp,
                    event.key_code,
                    event.event_type.as_str(),
                    modifiers_json,
                    event.application,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_events_in_range(&self, start: i64, end: i64) -> Result<Vec<KeystrokeEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, key_code, event_type, modifiers, application
             FROM keystroke_events
             WHERE timestamp >= ?1 AND timestamp <= ?2
             ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map(params![start, end], |row| {
            let event_type_str: String = row.get(2)?;
            let modifiers_json: String = row.get(3)?;

            Ok(KeystrokeEvent {
                timestamp: row.get(0)?,
                key_code: row.get(1)?,
                event_type: if event_type_str == "press" {
                    EventType::Press
                } else {
                    EventType::Release
                },
                modifiers: serde_json::from_str(&modifiers_json).unwrap_or_default(),
                application: row.get(4)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    pub fn get_all_events(&self) -> Result<Vec<KeystrokeEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, key_code, event_type, modifiers, application
             FROM keystroke_events
             ORDER BY timestamp ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            let event_type_str: String = row.get(2)?;
            let modifiers_json: String = row.get(3)?;

            Ok(KeystrokeEvent {
                timestamp: row.get(0)?,
                key_code: row.get(1)?,
                event_type: if event_type_str == "press" {
                    EventType::Press
                } else {
                    EventType::Release
                },
                modifiers: serde_json::from_str(&modifiers_json).unwrap_or_default(),
                application: row.get(4)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    pub fn get_events_since(&self, days_ago: u32) -> Result<Vec<KeystrokeEvent>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        
        let start = now - (days_ago as i64 * 24 * 60 * 60 * 1000);
        self.get_events_in_range(start, now)
    }

    pub fn get_total_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM keystroke_events", [], |row| {
                row.get(0)
            })?;
        Ok(count)
    }

    pub fn get_press_count(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM keystroke_events WHERE event_type = 'press'",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_date_range(&self) -> Result<Option<(i64, i64)>> {
        let result: Result<(i64, i64), _> = self.conn.query_row(
            "SELECT MIN(timestamp), MAX(timestamp) FROM keystroke_events",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok((min, max)) => Ok(Some((min, max))),
            Err(_) => Ok(None),
        }
    }

    pub fn get_top_keys(&self, limit: usize) -> Result<Vec<(u32, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT key_code, COUNT(*) as count
             FROM keystroke_events
             WHERE event_type = 'press'
             GROUP BY key_code
             ORDER BY count DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    pub fn get_top_applications(&self, limit: usize) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT application, COUNT(*) as count
             FROM keystroke_events
             WHERE event_type = 'press'
             GROUP BY application
             ORDER BY count DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    pub fn cleanup_old_events(&self, before_timestamp: i64) -> Result<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM keystroke_events WHERE timestamp < ?1",
            params![before_timestamp],
        )?;

        // These PRAGMAs return results, so use query_row and ignore the result
        let _ = self.conn.query_row("PRAGMA incremental_vacuum(100)", [], |_| Ok(()));
        let _ = self.conn.query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |_| Ok(()));

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::event::Modifier;

    fn create_test_event(timestamp: i64, key_code: u32, event_type: EventType) -> KeystrokeEvent {
        KeystrokeEvent {
            timestamp,
            key_code,
            event_type,
            modifiers: vec![],
            application: "com.test.app".to_string(),
        }
    }

    #[test]
    fn test_database_creation() {
        let db = Database::new(":memory:").unwrap();
        assert_eq!(db.get_total_count().unwrap(), 0);
    }

    #[test]
    fn test_insert_and_retrieve_event() {
        let db = Database::new(":memory:").unwrap();
        
        let event = create_test_event(1000, 0x00, EventType::Press);
        db.insert_event(&event).unwrap();

        assert_eq!(db.get_total_count().unwrap(), 1);
        assert_eq!(db.get_press_count().unwrap(), 1);
    }

    #[test]
    fn test_insert_multiple_events() {
        let db = Database::new(":memory:").unwrap();
        
        for i in 0..10 {
            let event = create_test_event(1000 + i, 0x00, EventType::Press);
            db.insert_event(&event).unwrap();
        }

        assert_eq!(db.get_total_count().unwrap(), 10);
    }

    #[test]
    fn test_press_vs_release_count() {
        let db = Database::new(":memory:").unwrap();
        
        db.insert_event(&create_test_event(1000, 0x00, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(1050, 0x00, EventType::Release)).unwrap();
        db.insert_event(&create_test_event(1100, 0x01, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(1150, 0x01, EventType::Release)).unwrap();

        assert_eq!(db.get_total_count().unwrap(), 4);
        assert_eq!(db.get_press_count().unwrap(), 2);
    }

    #[test]
    fn test_get_all_events() {
        let db = Database::new(":memory:").unwrap();
        
        db.insert_event(&create_test_event(1000, 0x00, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(1050, 0x00, EventType::Release)).unwrap();

        let events = db.get_all_events().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].timestamp, 1000);
        assert_eq!(events[1].timestamp, 1050);
    }

    #[test]
    fn test_get_events_in_range() {
        let db = Database::new(":memory:").unwrap();
        
        db.insert_event(&create_test_event(1000, 0x00, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(2000, 0x01, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(3000, 0x02, EventType::Press)).unwrap();

        let events = db.get_events_in_range(1500, 2500).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key_code, 0x01);
    }

    #[test]
    fn test_get_date_range() {
        let db = Database::new(":memory:").unwrap();
        
        db.insert_event(&create_test_event(1000, 0x00, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(5000, 0x01, EventType::Press)).unwrap();

        let range = db.get_date_range().unwrap().unwrap();
        assert_eq!(range, (1000, 5000));
    }

    #[test]
    fn test_get_top_keys() {
        let db = Database::new(":memory:").unwrap();
        
        for _ in 0..5 {
            db.insert_event(&create_test_event(1000, 0x00, EventType::Press)).unwrap();
        }
        for _ in 0..3 {
            db.insert_event(&create_test_event(1000, 0x01, EventType::Press)).unwrap();
        }
        db.insert_event(&create_test_event(1000, 0x02, EventType::Press)).unwrap();

        let top = db.get_top_keys(2).unwrap();
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], (0x00, 5));
        assert_eq!(top[1], (0x01, 3));
    }

    #[test]
    fn test_get_top_applications() {
        let db = Database::new(":memory:").unwrap();
        
        let mut event1 = create_test_event(1000, 0x00, EventType::Press);
        event1.application = "com.app.one".to_string();
        
        let mut event2 = create_test_event(1001, 0x00, EventType::Press);
        event2.application = "com.app.two".to_string();

        db.insert_event(&event1).unwrap();
        db.insert_event(&event1).unwrap();
        db.insert_event(&event2).unwrap();

        let top = db.get_top_applications(2).unwrap();
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "com.app.one");
        assert_eq!(top[0].1, 2);
    }

    #[test]
    fn test_cleanup_old_events() {
        let db = Database::new(":memory:").unwrap();
        
        db.insert_event(&create_test_event(1000, 0x00, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(2000, 0x01, EventType::Press)).unwrap();
        db.insert_event(&create_test_event(3000, 0x02, EventType::Press)).unwrap();

        let deleted = db.cleanup_old_events(2500).unwrap();
        assert_eq!(deleted, 2);
        assert_eq!(db.get_total_count().unwrap(), 1);
    }

    #[test]
    fn test_event_with_modifiers() {
        let db = Database::new(":memory:").unwrap();
        
        let event = KeystrokeEvent {
            timestamp: 1000,
            key_code: 0x00,
            event_type: EventType::Press,
            modifiers: vec![Modifier::Shift, Modifier::Command],
            application: "com.test.app".to_string(),
        };
        
        db.insert_event(&event).unwrap();
        
        let events = db.get_all_events().unwrap();
        assert_eq!(events[0].modifiers.len(), 2);
    }
}
