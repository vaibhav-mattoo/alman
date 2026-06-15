use rusqlite::Connection;
use std::env;
use std::fs;
use std::path::Path;

use super::db::now_secs;

/// If `events` is empty (fresh DB), load the shell history file and seed both
/// `events` and `command_stats`.  cwd/session/exit are NULL for bootstrapped rows.
pub fn bootstrap_from_history(conn: &Connection) {
    // Only seed when the database is empty
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
        .unwrap_or(0);
    if count > 0 {
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
    let interval = 120_i64; // 2 min spacing going backwards

    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(_) => return,
    };

    for (i, command) in commands.iter().enumerate() {
        let cmd = command.trim();
        if cmd.is_empty() || cmd.len() <= 2 {
            continue;
        }
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.len() <= 1 && cmd.len() <= 5 {
            continue;
        }
        if let Some(ref name) = binary_name {
            if parts.first().map(|w| *w == name.as_str()).unwrap_or(false) {
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

/// Insert or increment command_stats rows for a command and all its word-prefixes.
pub fn upsert_prefixes(conn: &Connection, full_cmd: &str, ts: i64) {
    let parts: Vec<&str> = full_cmd.split_whitespace().collect();
    let mut temp = String::new();

    for word in &parts {
        if !temp.is_empty() {
            temp.push(' ');
        }
        temp.push_str(word);

        let word_count = temp.split_whitespace().count();
        let length: i64 = temp.split_whitespace().map(|s| s.len()).sum::<usize>() as i64;
        if word_count == 1 && length <= 5 {
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
    }
}

// ---------------------------------------------------------------------------
// History file parsing (unchanged logic)
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
        if trimmed.is_empty() {
            continue;
        }
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
    // Zsh: ": 1234567890:0;command"
    if line.starts_with(": ") {
        if let Some(pos) = line.find(';') {
            let cmd = line[pos + 1..].trim();
            if !cmd.is_empty() {
                return cmd.to_string();
            }
        }
        return String::new();
    }

    // Fish: "- cmd:command"
    if let Some(stripped) = line.strip_prefix("- cmd:") {
        let cmd = stripped.trim();
        if !cmd.is_empty() {
            return cmd.to_string();
        }
        return String::new();
    }

    // Timestamp-only bash lines
    if line.starts_with('#') {
        return String::new();
    }

    line.to_string()
}
