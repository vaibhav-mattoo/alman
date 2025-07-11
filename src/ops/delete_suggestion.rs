use crate::database::database_structs::{Database,DeletedCommands};

pub fn delete_suggestion(alias: &str, db: &mut Database,deleted_commands: &mut DeletedCommands) {
    db.remove_command(&alias.to_string(), deleted_commands);
}
