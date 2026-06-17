use crate::ops::alias_ops::{
    add_alias_to_multiple_files, add_alias_to_multiple_files_force,
    get_aliases_from_multiple_files, remove_alias_from_multiple_files,
};
use rusqlite::Connection;

pub type AlmanError = Box<dyn std::error::Error>;

pub enum ApplyOutcome {
    Added { alias: String, command: String },
    Removed { alias: String },
    Changed { old_alias: String, new_alias: String, command: String },
    NotFound { alias: String },
}

/// Insert OR IGNORE into `dismissed`, then delete from `command_stats`.
fn dismiss_command(conn: &Connection, cmd: &str) -> Result<(), AlmanError> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT OR IGNORE INTO dismissed (command_text) VALUES (?1)",
        rusqlite::params![cmd],
    )?;
    tx.execute(
        "DELETE FROM command_stats WHERE command_text = ?1",
        rusqlite::params![cmd],
    )?;
    tx.commit()?;
    Ok(())
}

/// Remove from `dismissed` so the command can be suggested again.
fn undismiss_command(conn: &Connection, cmd: &str) -> Result<(), AlmanError> {
    conn.execute(
        "DELETE FROM dismissed WHERE command_text = ?1",
        rusqlite::params![cmd],
    )?;
    Ok(())
}

/// Add an alias across all files and dismiss its command from suggestions.
pub fn apply_add(
    conn: &Connection,
    paths: &[String],
    alias: &str,
    command: &str,
) -> Result<ApplyOutcome, AlmanError> {
    add_alias_to_multiple_files(paths, alias, command)?;
    dismiss_command(conn, command)?;
    Ok(ApplyOutcome::Added {
        alias: alias.to_string(),
        command: command.to_string(),
    })
}

/// Remove an alias from all files and un-dismiss its command.
///
/// Resolves the command *before* touching the files to avoid the
/// read-after-delete ordering bug.
pub fn apply_remove(
    conn: &Connection,
    paths: &[String],
    alias: &str,
) -> Result<ApplyOutcome, AlmanError> {
    let all = get_aliases_from_multiple_files(paths);
    let Some((_, command)) = all.into_iter().find(|(a, _)| a == alias) else {
        return Ok(ApplyOutcome::NotFound { alias: alias.to_string() });
    };
    remove_alias_from_multiple_files(paths, alias)?;
    undismiss_command(conn, &command)?;
    Ok(ApplyOutcome::Removed { alias: alias.to_string() })
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
) -> Result<ApplyOutcome, AlmanError> {
    let all = get_aliases_from_multiple_files(paths);
    let Some((_, command)) = all.into_iter().find(|(a, _)| a == old_alias) else {
        return Ok(ApplyOutcome::NotFound { alias: old_alias.to_string() });
    };
    remove_alias_from_multiple_files(paths, old_alias)?;
    // Briefly un-dismiss so force-add goes through cleanly, then re-dismiss.
    undismiss_command(conn, &command)?;
    add_alias_to_multiple_files_force(paths, new_alias, &command)?;
    dismiss_command(conn, &command)?;
    Ok(ApplyOutcome::Changed {
        old_alias: old_alias.to_string(),
        new_alias: new_alias.to_string(),
        command,
    })
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

        apply_add(&conn, &paths(&path), "ga", "git add -A").unwrap();

        let dismissed: bool = conn
            .query_row(
                "SELECT 1 FROM dismissed WHERE command_text = 'git add -A'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(dismissed, "command not dismissed after apply_add");

        apply_remove(&conn, &paths(&path), "ga").unwrap();

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
        let outcome = apply_remove(&conn, &paths(&path), "nonexistent").unwrap();
        assert!(matches!(outcome, ApplyOutcome::NotFound { .. }));
    }

    #[test]
    fn change_replaces_alias_and_keeps_command_dismissed() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let path = tmp.path().to_string_lossy().to_string();
        let conn = open_mem();

        apply_add(&conn, &paths(&path), "ga", "git add -A").unwrap();
        apply_change(&conn, &paths(&path), "ga", "gaa").unwrap();

        let aliases = get_aliases_from_multiple_files(&paths(&path));
        assert!(aliases.iter().any(|(a, _)| a == "gaa"), "new alias missing");
        assert!(!aliases.iter().any(|(a, _)| a == "ga"), "old alias still present");

        let dismissed: bool = conn
            .query_row(
                "SELECT 1 FROM dismissed WHERE command_text = 'git add -A'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(dismissed, "command should still be dismissed after change");
    }

    #[test]
    fn add_error_propagates_not_silenced() {
        let conn = open_mem();
        // Pass a path in a nonexistent directory — should return Err, not panic.
        let bad_paths = vec!["/nonexistent/dir/aliases".to_string()];
        let result = apply_add(&conn, &bad_paths, "gs", "git status");
        assert!(result.is_err(), "expected Err on bad path");
    }
}
