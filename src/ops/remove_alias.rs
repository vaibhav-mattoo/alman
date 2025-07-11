use crate::database::database_structs::{DeletedCommands};
use crate::ops::alias_ops::{remove_alias_from_file, get_aliases};

pub fn remove_alias(deleted_commands: &mut DeletedCommands, file_path: &str, alias: &str) {
    // we are removing an alias, so we need to remove from deleted commands, 
    // so that future commands can be added
    // for this we simply remove the command from deleted commands
    let list = get_aliases(file_path);
    // find command for the alias
    if let Some((_, command)) = list.iter().find(|(a, _)| a == alias) {
        // remove the command from deleted commands
        deleted_commands.deleted_commands.remove(command);
    }

    // then remove from the file.
    remove_alias_from_file(file_path, alias);

}

