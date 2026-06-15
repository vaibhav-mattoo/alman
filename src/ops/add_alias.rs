use crate::ops::alias_ops::add_alias_to_file;
use rusqlite::Connection;

/// Write the alias to the alias file, then dismiss the command from suggestions
/// so it no longer appears in get-suggestions output.
pub fn add_alias(conn: &Connection, file_path: &str, alias: &str, command: &str) {
    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(e) => { eprintln!("alman: DB error: {e}"); return; }
    };
    let _ = tx.execute(
        "INSERT OR IGNORE INTO dismissed (command_text) VALUES (?1)",
        rusqlite::params![command],
    );
    let _ = tx.execute(
        "DELETE FROM command_stats WHERE command_text = ?1",
        rusqlite::params![command],
    );
    let _ = tx.commit();

    add_alias_to_file(file_path, alias, command);
}
