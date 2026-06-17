use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::Path;

use super::db::now_secs;

const MAX_PREFIX_WORDS: usize = 3;
const COMPACT_STALE_SECS: i64 = 90 * 86_400;   // 90 days
const COMPACT_EVENTS_SECS: i64 = 365 * 86_400;  // 365 days

/// Seed `events` and `command_stats` from the user's shell history on a fresh DB.
/// Early-returns unless BOTH tables are empty (guards against double-seeding on migration).
pub fn bootstrap_from_history(conn: &Connection) {
    let events_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
        .unwrap_or(0);
    let stats_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM command_stats", [], |r| r.get(0))
        .unwrap_or(0);
    if events_count > 0 || stats_count > 0 {
        return;
    }

    let history_file = match get_history_file_path() {
        Ok(f) => f,
        Err(_) => return,
    };
    if !Path::new(&history_file).exists() {
        return;
    }

    let content = match fs::read_to_string(&history_file) {
        Ok(c) => c,
        Err(_) => match fs::read(&history_file) {
            Ok(b) => String::from_utf8_lossy(&b).into_owned(),
            Err(_) => return,
        },
    };

    let commands = parse_history_file(&content);
    if commands.is_empty() {
        return;
    }

    let binary_name = env::args()
        .next()
        .and_then(|p| Path::new(&p).file_name().map(|f| f.to_os_string()))
        .and_then(|s| s.into_string().ok());

    let base_ts = now_secs();
    let interval = 120_i64;

    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(_) => return,
    };

    for (i, command) in commands.iter().enumerate() {
        let cmd = command.trim();
        if cmd.is_empty() || cmd.len() <= 2 {
            continue;
        }
        let parts: Vec<String> = crate::defaults::default_tokenizer().tokenize(cmd);
        if parts.len() <= 1 && cmd.len() <= 5 {
            continue;
        }
        if let Some(ref name) = binary_name {
            if parts.first().map(|w| w == name.as_str()).unwrap_or(false) {
                continue;
            }
        }

        let ts = base_ts - (i as i64 * interval);

        let _ = tx.execute(
            "INSERT INTO events (command, ts, session_id, cwd, exit_code) VALUES (?1, ?2, NULL, NULL, NULL)",
            rusqlite::params![cmd, ts],
        );

        upsert_prefixes(&tx, cmd, ts);
    }

    let _ = tx.commit();
}

/// Insert or increment `command_stats` rows for a command and its word-prefixes.
///
/// Stops before the first flag token (starting with `-`) and caps at
/// `MAX_PREFIX_WORDS` words so quoted arguments never produce junk rows.
pub fn upsert_prefixes(conn: &Connection, full_cmd: &str, ts: i64) {
    let parts: Vec<String> = crate::defaults::default_tokenizer().tokenize(full_cmd);
    let mut temp = String::new();
    let mut word_count = 0usize;

    for word in &parts {
        if word.starts_with('-') {
            break;
        }

        if !temp.is_empty() {
            temp.push(' ');
        }
        temp.push_str(word);
        word_count += 1;

        let length: i64 = temp.split_whitespace().map(|s| s.len()).sum::<usize>() as i64;
        if word_count == 1 && length <= 5 {
            if word_count >= MAX_PREFIX_WORDS { break; }
            continue;
        }

        let _ = conn.execute(
            "INSERT INTO command_stats (command_text, frequency, last_access_time, length)
             SELECT ?1, 1, ?2, ?3
             WHERE ?1 NOT IN (SELECT command_text FROM dismissed)
             ON CONFLICT(command_text) DO UPDATE SET
               frequency        = frequency + 1,
               last_access_time = excluded.last_access_time",
            rusqlite::params![temp, ts, length],
        );

        if word_count >= MAX_PREFIX_WORDS {
            break;
        }
    }
}

/// Delete stale low-value `command_stats` rows and prune old `events`.
/// Call only from infrequent entry points (init-data, TUI launch).
pub fn compact(conn: &Connection) {
    let now = now_secs();
    let _ = conn.execute(
        "DELETE FROM command_stats WHERE frequency <= 1 AND last_access_time < ?1",
        rusqlite::params![now - COMPACT_STALE_SECS],
    );
    let _ = conn.execute(
        "DELETE FROM events WHERE ts < ?1",
        rusqlite::params![now - COMPACT_EVENTS_SECS],
    );
}

// ---------------------------------------------------------------------------
// History file parsing
// ---------------------------------------------------------------------------

fn get_history_file_path() -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(histfile) = env::var("HISTFILE") {
        if !histfile.is_empty() {
            return Ok(histfile);
        }
    }

    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
    let possible_paths = vec![
        home_dir.join(".local/share/fish/fish_history"),
        home_dir.join(".zsh_history"),
        home_dir.join(".bash_history"),
        home_dir.join(".history"),
        home_dir.join(".fish_history"),
    ];

    for path in possible_paths {
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    Err("No history file found".into())
}

fn parse_history_file(content: &str) -> Vec<String> {
    let mut commands = Vec::new();

    for line in content.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if trimmed.starts_with('#')
            || trimmed.starts_with("HISTTIMEFORMAT")
            || trimmed.starts_with("HISTSIZE")
            || trimmed.starts_with("HISTFILESIZE")
        {
            continue;
        }

        let command = extract_command(trimmed);
        if !command.is_empty() && command.len() > 2 {
            commands.push(command);
        }
    }

    commands
}

fn extract_command(line: &str) -> String {
    if line.starts_with(": ") {
        if let Some(pos) = line.find(';') {
            let cmd = line[pos + 1..].trim();
            if !cmd.is_empty() { return cmd.to_string(); }
        }
        return String::new();
    }

    if let Some(stripped) = line.strip_prefix("- cmd:") {
        let cmd = stripped.trim();
        if !cmd.is_empty() { return cmd.to_string(); }
        return String::new();
    }

    if line.starts_with('#') {
        return String::new();
    }

    line.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::db;

    /// Open an in-memory DB with schema but WITHOUT bootstrap or migration.
    fn open_raw() -> Connection {
        db::open_for_write(":memory:").expect("in-memory DB")
    }

    #[test]
    fn upsert_prefixes_stops_at_flag() {
        let conn = open_raw();
        upsert_prefixes(&conn, "git commit -m 'fix bug'", 1_000_000);

        let rows: Vec<String> = {
            let mut stmt = conn.prepare("SELECT command_text FROM command_stats").unwrap();
            stmt.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect()
        };
        // Should have "git commit" only — stops before "-m"
        assert!(rows.iter().any(|r| r == "git commit"), "expected 'git commit'");
        // No word in any row should start with '-' (flag tokens must not be stored)
        for row in &rows {
            for word in row.split_whitespace() {
                assert!(!word.starts_with('-'),
                    "flag-like token '{}' stored in row '{}'", word, row);
            }
        }
        assert!(!rows.iter().any(|r| r.split_whitespace().any(|w| w == "'fix")),
            "quoted arg must not appear as separate token");
    }

    #[test]
    fn upsert_prefixes_caps_at_max_words() {
        let conn = open_raw();
        upsert_prefixes(&conn, "one two three four five", 1_000_000);

        let rows: Vec<String> = {
            let mut stmt = conn.prepare("SELECT command_text FROM command_stats").unwrap();
            stmt.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect()
        };
        // Max 3 words; "one" is skipped (single word ≤5 chars)
        for row in &rows {
            let wc = row.split_whitespace().count();
            assert!(wc <= MAX_PREFIX_WORDS, "prefix '{}' exceeds MAX_PREFIX_WORDS", row);
        }
        assert!(!rows.iter().any(|r| r == "one two three four"), "4-word prefix must not exist");
    }

    #[test]
    fn bootstrap_guard_skips_when_stats_populated() {
        // Use a raw connection (no bootstrap) so we control initial state.
        let conn = open_raw();
        // Seed command_stats (simulating migration)
        conn.execute(
            "INSERT INTO command_stats (command_text, frequency, last_access_time, length) \
             VALUES ('git status', 1, 0, 10)",
            [],
        ).unwrap();
        // bootstrap must return early because command_stats is non-empty
        bootstrap_from_history(&conn);
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0, "bootstrap must not seed when command_stats is non-empty");
    }
}
