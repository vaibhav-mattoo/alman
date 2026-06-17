use crate::registry::{self, DefinitionKind};
use crate::template::{CommandTemplate, TemplatePart};
use rusqlite::Connection;

pub type AlmanError = Box<dyn std::error::Error>;

pub enum ApplyOutcome {
    Added { name: String },
    Removed,
    Changed,
    NotFound { name: String },
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

/// Add a plain alias: template = [Literal(command)].
pub fn apply_add(conn: &Connection, name: &str, command: &str) -> Result<ApplyOutcome, AlmanError> {
    let template = CommandTemplate {
        parts: vec![TemplatePart::Literal(command.to_string())],
    };
    registry::upsert_definition(conn, name, DefinitionKind::Alias, &template)?;
    dismiss_command(conn, command)?;
    Ok(ApplyOutcome::Added {
        name: name.to_string(),
    })
}

/// Add a parameterized function (or alias if the shape allows it).
pub fn apply_add_function(
    conn: &Connection,
    name: &str,
    template: &CommandTemplate,
) -> Result<ApplyOutcome, AlmanError> {
    let kind = if template.is_zero_slot() || template.only_trailing_single_slot() {
        DefinitionKind::Alias
    } else {
        DefinitionKind::Function
    };
    registry::upsert_definition(conn, name, kind, template)?;
    // Dismiss the literal skeleton (literal tokens only, for command_stats).
    let skeleton: String = template
        .parts
        .iter()
        .filter_map(|p| {
            if let TemplatePart::Literal(s) = p {
                Some(s.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    if !skeleton.is_empty() {
        let _ = dismiss_command(conn, &skeleton);
    }
    Ok(ApplyOutcome::Added {
        name: name.to_string(),
    })
}

/// Remove by name; un-dismiss its literal command if it was a zero-slot alias.
pub fn apply_remove(conn: &Connection, name: &str) -> Result<ApplyOutcome, AlmanError> {
    let defs = registry::list_definitions(conn)?;
    let def = defs.iter().find(|d| d.name == name);
    let literal_cmd = def.and_then(|d| {
        if d.template.is_zero_slot() {
            if let Some(TemplatePart::Literal(s)) = d.template.parts.first() {
                return Some(s.clone());
            }
        }
        None
    });
    if !registry::remove_definition(conn, name)? {
        return Ok(ApplyOutcome::NotFound {
            name: name.to_string(),
        });
    }
    if let Some(cmd) = literal_cmd {
        let _ = undismiss_command(conn, &cmd);
    }
    Ok(ApplyOutcome::Removed)
}

/// Rename a definition.
pub fn apply_change(
    conn: &Connection,
    old_name: &str,
    new_name: &str,
) -> Result<ApplyOutcome, AlmanError> {
    if !registry::rename_definition(conn, old_name, new_name)? {
        return Ok(ApplyOutcome::NotFound {
            name: old_name.to_string(),
        });
    }
    Ok(ApplyOutcome::Changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::db;

    fn open_mem() -> Connection {
        db::open(":memory:").expect("in-memory DB")
    }

    #[test]
    fn add_creates_definition_and_dismisses() {
        let conn = open_mem();
        apply_add(&conn, "ga", "git add -A").unwrap();

        assert!(registry::definition_exists(&conn, "ga").unwrap());
        let dismissed: bool = conn
            .query_row(
                "SELECT 1 FROM dismissed WHERE command_text = 'git add -A'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        assert!(dismissed, "command not dismissed after apply_add");
    }

    #[test]
    fn remove_undismisses_command() {
        let conn = open_mem();
        apply_add(&conn, "ga", "git add -A").unwrap();
        apply_remove(&conn, "ga").unwrap();

        assert!(!registry::definition_exists(&conn, "ga").unwrap());
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
        let conn = open_mem();
        let outcome = apply_remove(&conn, "nonexistent").unwrap();
        assert!(matches!(outcome, ApplyOutcome::NotFound { .. }));
    }

    #[test]
    fn change_renames_definition() {
        let conn = open_mem();
        apply_add(&conn, "ga", "git add -A").unwrap();
        apply_change(&conn, "ga", "gaa").unwrap();

        assert!(!registry::definition_exists(&conn, "ga").unwrap());
        assert!(registry::definition_exists(&conn, "gaa").unwrap());
    }

    #[test]
    fn add_function_with_interior_slot_is_function() {
        let conn = open_mem();
        let template = CommandTemplate {
            parts: vec![
                TemplatePart::Literal("docker".into()),
                TemplatePart::Literal("exec".into()),
                TemplatePart::Slot(1),
                TemplatePart::Literal("bash".into()),
            ],
        };
        apply_add_function(&conn, "dex", &template).unwrap();
        let defs = registry::list_definitions(&conn).unwrap();
        let d = defs.iter().find(|d| d.name == "dex").unwrap();
        assert_eq!(d.kind, DefinitionKind::Function);
    }

    #[test]
    fn add_function_with_trailing_slot_is_alias() {
        let conn = open_mem();
        let template = CommandTemplate {
            parts: vec![TemplatePart::Literal("git".into()), TemplatePart::Slot(1)],
        };
        apply_add_function(&conn, "g", &template).unwrap();
        let defs = registry::list_definitions(&conn).unwrap();
        let d = defs.iter().find(|d| d.name == "g").unwrap();
        assert_eq!(d.kind, DefinitionKind::Alias);
    }
}
