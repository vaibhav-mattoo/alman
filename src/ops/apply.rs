use crate::ops::alias_ops::{
    add_alias_to_multiple_files, add_alias_to_multiple_files_force,
    get_aliases_from_multiple_files, remove_alias_from_multiple_files,
};
use rusqlite::Connection;

#[allow(dead_code)]
pub enum ApplyOutcome {
    Added { alias: String, command: String },
    Removed { alias: String },
    Changed { old_alias: String, new_alias: String, command: String },
    NotFound { alias: String },
}

/// Insert OR IGNORE into `dismissed`, then delete from `command_stats`.
fn dismiss_command(conn: &Connection, cmd: &str) {
    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(e) => { eprintln!("alman: DB error: {e}"); return; }
    };
    let _ = tx.execute(
        "INSERT OR IGNORE INTO dismissed (command_text) VALUES (?1)",
        rusqlite::params![cmd],
    );
    let _ = tx.execute(
        "DELETE FROM command_stats WHERE command_text = ?1",
        rusqlite::params![cmd],
    );
    let _ = tx.commit();
}

/// Remove from `dismissed` so the command can be suggested again.
fn undismiss_command(conn: &Connection, cmd: &str) {
    let _ = conn.execute(
        "DELETE FROM dismissed WHERE command_text = ?1",
        rusqlite::params![cmd],
    );
}

/// Add an alias across all files and dismiss its command from suggestions.
pub fn apply_add(
    conn: &Connection,
    paths: &[String],
    alias: &str,
    command: &str,
) -> ApplyOutcome {
    add_alias_to_multiple_files(paths, alias, command);
    dismiss_command(conn, command);
    ApplyOutcome::Added {
        alias: alias.to_string(),
        command: command.to_string(),
    }
}

/// Remove an alias from all files and un-dismiss its command.
///
/// Resolves the command *before* touching the files to avoid the
/// read-after-delete ordering bug.
pub fn apply_remove(
    conn: &Connection,
    paths: &[String],
    alias: &str,
) -> ApplyOutcome {
    let all = get_aliases_from_multiple_files(paths);
    let Some((_, command)) = all.into_iter().find(|(a, _)| a == alias) else {
        return ApplyOutcome::NotFound { alias: alias.to_string() };
    };
    remove_alias_from_multiple_files(paths, alias);
    undismiss_command(conn, &command);
    ApplyOutcome::Removed { alias: alias.to_string() }
}

/// Rename an alias: remove old, add new, keep the command dismissed.
///
/// Resolves the command *before* touching the files to avoid the
/// read-after-delete ordering bug.
pub fn apply_change(
    conn: &Connection,
    paths: &[String],
    old_alias: &str,
    new_alias: &str,
) -> ApplyOutcome {
    let all = get_aliases_from_multiple_files(paths);
    let Some((_, command)) = all.into_iter().find(|(a, _)| a == old_alias) else {
        return ApplyOutcome::NotFound { alias: old_alias.to_string() };
    };
    remove_alias_from_multiple_files(paths, old_alias);
    // Briefly un-dismiss so force-add goes through cleanly, then re-dismiss.
    undismiss_command(conn, &command);
    add_alias_to_multiple_files_force(paths, new_alias, &command);
    dismiss_command(conn, &command);
    ApplyOutcome::Changed {
        old_alias: old_alias.to_string(),
        new_alias: new_alias.to_string(),
        command,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::db;

    fn open_mem() -> Connection {
        db::open(":memory:").expect("in-memory DB")
    }

    fn paths(p: &str) -> Vec<String> {
        vec![p.to_string()]
    }

    #[test]
    fn remove_undismisses_command() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let conn = open_mem();

        // Set up: alias ga='git add -A' and dismiss the command.
        apply_add(&conn, &paths(&path), "ga", "git add -A");

        // Sanity: command should be dismissed now.
        let dismissed: bool = conn
            .query_row(
                "SELECT 1 FROM dismissed WHERE command_text = 'git add -A'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(dismissed, "command not dismissed after apply_add");

        // Now remove the alias — the command should be un-dismissed.
        apply_remove(&conn, &paths(&path), "ga");

        let still_dismissed: bool = conn
            .query_row(
                "SELECT 1 FROM dismissed WHERE command_text = 'git add -A'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(!still_dismissed, "command still dismissed after apply_remove");
    }

    #[test]
    fn remove_not_found_returns_not_found() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let conn = open_mem();
        let outcome = apply_remove(&conn, &paths(&path), "nonexistent");
        assert!(matches!(outcome, ApplyOutcome::NotFound { .. }));
    }

    #[test]
    fn change_replaces_alias_and_keeps_command_dismissed() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let conn = open_mem();

        apply_add(&conn, &paths(&path), "ga", "git add -A");
        apply_change(&conn, &paths(&path), "ga", "gaa");

        // New alias present, old alias gone from file.
        let aliases = get_aliases_from_multiple_files(&paths(&path));
        assert!(aliases.iter().any(|(a, _)| a == "gaa"), "new alias missing");
        assert!(!aliases.iter().any(|(a, _)| a == "ga"), "old alias still present");

        // Command still dismissed under the new alias.
        let dismissed: bool = conn
            .query_row(
                "SELECT 1 FROM dismissed WHERE command_text = 'git add -A'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(dismissed, "command should still be dismissed after change");
    }
}
