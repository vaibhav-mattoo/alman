use crate::ops::alias_ops::{get_aliases, remove_alias_from_file};
use rusqlite::Connection;

/// Remove the alias from the alias file, then un-dismiss its command so it can
/// be suggested again.
pub fn remove_alias(conn: &Connection, file_path: &str, alias: &str) {
    let list = get_aliases(file_path);
    if let Some((_, command)) = list.iter().find(|(a, _)| a == alias) {
        let _ = conn.execute(
            "DELETE FROM dismissed WHERE command_text = ?1",
            rusqlite::params![command],
        );
    }
    remove_alias_from_file(file_path, alias);
}
