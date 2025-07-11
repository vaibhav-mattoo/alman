use crate::database::database_structs::{Database,DeletedCommands};
use crate::ops::alias_ops::{add_alias_to_file};

pub fn add_alias(db: &mut Database, deleted_commands: &mut DeletedCommands, file_path: &str, alias: &str, command: &str) {
    // first add it to deleted commands
    // use wrapper from database_ops.rs
    db.remove_command(&command.to_string(), deleted_commands);

    // second add to file.
    add_alias_to_file(file_path, alias, command);
}

