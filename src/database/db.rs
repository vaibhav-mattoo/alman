use rusqlite::{Connection, Result, functions::FunctionFlags};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn get_db_path() -> String {
    crate::database::persistence::get_data_directory()
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local")
                .join("share")
                .join("alman")
        })
        .join("alman.db")
        .to_string_lossy()
        .to_string()
}

/// Lightweight open for the `custom` hot-write path.
/// Sets pragmas and ensures the schema exists. No UDF, no migration, no bootstrap.
pub fn open_for_write(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA busy_timeout=3000;
         PRAGMA foreign_keys=ON;",
    )?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS events (
             id         INTEGER PRIMARY KEY,
             command    TEXT    NOT NULL,
             ts         INTEGER NOT NULL,
             session_id TEXT,
             cwd        TEXT,
             exit_code  INTEGER
         );
         CREATE INDEX IF NOT EXISTS idx_events_session_ts ON events(session_id, ts);

         CREATE TABLE IF NOT EXISTS command_stats (
             command_text     TEXT    PRIMARY KEY,
             frequency        INTEGER NOT NULL,
             last_access_time INTEGER NOT NULL,
             length           INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS dismissed (
             command_text TEXT PRIMARY KEY
         );",
    )?;
    Ok(conn)
}

/// Full open: pragmas + schema + UDF + one-time migration + bootstrap on fresh DB.
pub fn open(path: &str) -> Result<Connection> {
    let is_new = !std::path::Path::new(path).exists();

    let conn = Connection::open(path)?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA busy_timeout=3000;
         PRAGMA foreign_keys=ON;",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS events (
             id         INTEGER PRIMARY KEY,
             command    TEXT    NOT NULL,
             ts         INTEGER NOT NULL,
             session_id TEXT,
             cwd        TEXT,
             exit_code  INTEGER
         );
         CREATE INDEX IF NOT EXISTS idx_events_session_ts ON events(session_id, ts);

         CREATE TABLE IF NOT EXISTS command_stats (
             command_text     TEXT    PRIMARY KEY,
             frequency        INTEGER NOT NULL,
             last_access_time INTEGER NOT NULL,
             length           INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS dismissed (
             command_text TEXT PRIMARY KEY
         );",
    )?;

    // alman_score(frequency, last_access_time, length, now) -> f64
    conn.create_scalar_function(
        "alman_score",
        4,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        |ctx| {
            let frequency: f64 = ctx.get::<i64>(0)? as f64;
            let last_access: i64 = ctx.get::<i64>(1)?;
            let length: f64 = ctx.get::<i64>(2)? as f64;
            let now: i64 = ctx.get::<i64>(3)?;
            Ok(crate::database::scoring::score(frequency, last_access, length, now))
        },
    )?;

    if is_new {
        migrate_from_bincode(&conn);
        crate::database::history_loader::bootstrap_from_history(&conn);
    }

    Ok(conn)
}

// ---------------------------------------------------------------------------
// One-time migration from bincode files
// ---------------------------------------------------------------------------

fn migrate_from_bincode(conn: &Connection) {
    use legacy::{Database, DeletedCommands};
    use std::fs;

    let data_dir = match crate::database::persistence::get_data_directory() {
        Ok(d) => d,
        Err(_) => return,
    };

    let db_bin = data_dir.join("command_database.bin");
    let dc_bin = data_dir.join("deleted_commands.bin");

    if !db_bin.exists() && !dc_bin.exists() {
        return;
    }

    eprintln!("alman: migrating from bincode to SQLite …");

    if dc_bin.exists() {
        if let Ok(bytes) = fs::read(&dc_bin) {
            if let Ok(dc) = bincode::deserialize::<DeletedCommands>(&bytes) {
                let tx = match conn.unchecked_transaction() {
                    Ok(t) => t,
                    Err(e) => { eprintln!("  migration error (dismissed tx): {e}"); return; }
                };
                for cmd in &dc.deleted_commands {
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO dismissed (command_text) VALUES (?1)",
                        rusqlite::params![cmd],
                    );
                }
                let _ = tx.commit();
            }
        }
        let _ = fs::rename(&dc_bin, dc_bin.with_extension("bin.bak"));
    }

    if db_bin.exists() {
        if let Ok(bytes) = fs::read(&db_bin) {
            if let Ok(db) = bincode::deserialize::<Database>(&bytes) {
                let tx = match conn.unchecked_transaction() {
                    Ok(t) => t,
                    Err(e) => { eprintln!("  migration error (stats tx): {e}"); return; }
                };
                for cmd in db.reverse_command_map.values() {
                    if cmd.length <= 5 && cmd.number_of_words == 1 {
                        continue;
                    }
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO command_stats
                             (command_text, frequency, last_access_time, length)
                         VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![
                            cmd.command_text,
                            cmd.frequency as i64,
                            cmd.last_access_time,
                            cmd.length as i64,
                        ],
                    );
                }
                let _ = tx.commit();
            }
        }
        let _ = fs::rename(&db_bin, db_bin.with_extension("bin.bak"));
    }

    eprintln!("alman: migration done — old .bin files renamed to .bin.bak");
}

// Structs kept only for deserializing the legacy bincode files.
mod legacy {
    use serde::{Deserialize, Serialize};
    use std::collections::{BTreeSet, HashMap};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Database {
        pub command_list: BTreeSet<Command>,
        pub reverse_command_map: HashMap<String, Command>,
        pub total_num_commands: i32,
        pub total_score: i64,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct DeletedCommands {
        pub deleted_commands: BTreeSet<String>,
    }

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct Command {
        pub score: i32,
        pub last_access_time: i64,
        pub frequency: i32,
        pub length: i16,
        pub command_text: String,
        pub number_of_words: i8,
    }

    impl Ord for Command {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match other.score.cmp(&self.score) {
                std::cmp::Ordering::Equal => self.command_text.cmp(&other.command_text),
                ord => ord,
            }
        }
    }
    impl PartialOrd for Command {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
}
