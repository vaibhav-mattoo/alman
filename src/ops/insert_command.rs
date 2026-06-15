use crate::database::db::now_secs;
use crate::database::history_loader::upsert_prefixes;
use rusqlite::Connection;

pub fn insert_command(
    command_str: String,
    conn: &Connection,
    session_id: Option<&str>,
    cwd: Option<&str>,
) {
    let command_str = command_str.trim().to_string();
    if command_str.is_empty() {
        return;
    }

    let parts: Vec<&str> = command_str.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    // Skip commands that start with the alman binary itself
    let binary_name = std::env::args()
        .next()
        .and_then(|p| std::path::Path::new(&p).file_name().map(|f| f.to_os_string()))
        .and_then(|s| s.into_string().ok());
    if let Some(ref name) = binary_name {
        if parts[0] == name.as_str() {
            return;
        }
    }

    let now = now_secs();

    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("alman: DB error starting transaction: {e}");
            return;
        }
    };

    if let Err(e) = tx.execute(
        "INSERT INTO events (command, ts, session_id, cwd, exit_code) VALUES (?1, ?2, ?3, ?4, NULL)",
        rusqlite::params![command_str, now, session_id, cwd],
    ) {
        eprintln!("alman: DB error inserting event: {e}");
        return;
    }

    upsert_prefixes(&tx, &command_str, now);

    if let Err(e) = tx.commit() {
        eprintln!("alman: DB error committing: {e}");
    }
}
