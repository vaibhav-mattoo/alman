use rusqlite::Connection;

pub fn delete_suggestion(command_text: &str, conn: &Connection) {
    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(e) => { eprintln!("alman: DB error: {e}"); return; }
    };
    let _ = tx.execute(
        "INSERT OR IGNORE INTO dismissed (command_text) VALUES (?1)",
        rusqlite::params![command_text],
    );
    let _ = tx.execute(
        "DELETE FROM command_stats WHERE command_text = ?1",
        rusqlite::params![command_text],
    );
    let _ = tx.commit();
}
