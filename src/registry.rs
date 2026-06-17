use crate::database::db::now_secs;
use crate::ops::apply::AlmanError;
use crate::template::CommandTemplate;
use rusqlite::{params, Connection};

#[derive(Debug, Clone, PartialEq)]
pub enum DefinitionKind {
    Alias,
    Function,
}

impl DefinitionKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Alias => "alias",
            Self::Function => "function",
        }
    }
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "alias" => Some(Self::Alias),
            "function" => Some(Self::Function),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: String,
    pub kind: DefinitionKind,
    pub template: CommandTemplate,
}

pub fn upsert_definition(
    conn: &Connection,
    name: &str,
    kind: DefinitionKind,
    template: &CommandTemplate,
) -> Result<(), AlmanError> {
    conn.execute(
        "INSERT INTO definitions (name, kind, template_json, created_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(name) DO UPDATE SET kind=excluded.kind, template_json=excluded.template_json",
        params![name, kind.as_str(), template.to_json(), now_secs()],
    )?;
    Ok(())
}

pub fn remove_definition(conn: &Connection, name: &str) -> Result<bool, AlmanError> {
    let n = conn.execute("DELETE FROM definitions WHERE name = ?1", params![name])?;
    Ok(n > 0)
}

pub fn rename_definition(conn: &Connection, old: &str, new: &str) -> Result<bool, AlmanError> {
    let n = conn.execute(
        "UPDATE definitions SET name = ?2 WHERE name = ?1",
        params![old, new],
    )?;
    Ok(n > 0)
}

pub fn list_definitions(conn: &Connection) -> Result<Vec<Definition>, AlmanError> {
    let mut stmt = conn
        .prepare("SELECT name, kind, template_json FROM definitions ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let kind_str: String = row.get(1)?;
        let tmpl_json: String = row.get(2)?;
        Ok((name, kind_str, tmpl_json))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (name, kind_str, tmpl_json) = row?;
        let kind = DefinitionKind::from_str(&kind_str).ok_or("bad kind")?;
        let template = CommandTemplate::from_json(&tmpl_json).ok_or("bad template")?;
        out.push(Definition { name, kind, template });
    }
    Ok(out)
}

#[cfg(test)]
pub fn definition_exists(conn: &Connection, name: &str) -> Result<bool, AlmanError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM definitions WHERE name = ?1",
        params![name],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}
