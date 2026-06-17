use crate::database::database_structs::Command;
use crate::database::db::now_secs;
use crate::ops::alias_suggestions::{AliasSuggester, AliasSuggestion};
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct CommandWithAlias {
    pub command: Command,
    pub alias_suggestions: Vec<AliasSuggestion>,
}

pub fn get_suggestions_with_aliases(
    num: Option<usize>,
    conn: &Connection,
    alias_file_path: &str,
) -> Vec<CommandWithAlias> {
    let limit = num.unwrap_or(5) as i64;
    let now = now_secs();

    let commands = query_top_commands(conn, now, limit);
    let paths = vec![alias_file_path.to_string()];
    let suggester = AliasSuggester::new(&paths);

    commands
        .into_iter()
        .map(|cmd| {
            let alias_suggestions = suggester.suggest_aliases(&cmd.command_text);
            CommandWithAlias { command: cmd, alias_suggestions }
        })
        .collect()
}

/// Fetch the top `limit` commands by score, excluding dismissed entries.
pub fn query_top_commands(conn: &Connection, now: i64, limit: i64) -> Vec<Command> {
    query_filtered_commands(conn, now, "", limit)
}

/// Fetch up to `limit` commands matching `filter` (case-insensitive substring),
/// ordered by score, excluding dismissed entries.
pub fn query_filtered_commands(conn: &Connection, now: i64, filter: &str, limit: i64) -> Vec<Command> {
    let sql = if filter.is_empty() {
        "SELECT command_text, frequency, last_access_time, length,
                alman_score(frequency, last_access_time, length, ?1) AS score
         FROM command_stats
         WHERE command_text NOT IN (SELECT command_text FROM dismissed)
         ORDER BY score DESC
         LIMIT ?2"
    } else {
        "SELECT command_text, frequency, last_access_time, length,
                alman_score(frequency, last_access_time, length, ?1) AS score
         FROM command_stats
         WHERE command_text NOT IN (SELECT command_text FROM dismissed)
           AND LOWER(command_text) LIKE '%' || LOWER(?3) || '%'
         ORDER BY score DESC
         LIMIT ?2"
    };

    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => { eprintln!("alman: prepare error: {e}"); return vec![]; }
    };

    let map_row = |row: &rusqlite::Row<'_>| {
        Ok(Command {
            command_text:     row.get(0)?,
            frequency:        row.get(1)?,
            last_access_time: row.get(2)?,
            length:           row.get(3)?,
            score:            row.get(4)?,
        })
    };

    let query_result = if filter.is_empty() {
        stmt.query_map(rusqlite::params![now, limit], map_row)
    } else {
        stmt.query_map(rusqlite::params![now, limit, filter], map_row)
    };

    match query_result {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => { eprintln!("alman: query error: {e}"); vec![] }
    }
}
