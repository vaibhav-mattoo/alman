use crate::database::database_structs::{Command, Database};
use crate::ops::alias_suggestions::{AliasSuggester, AliasSuggestion};

#[derive(Debug, Clone)]
pub struct CommandWithAlias {
    pub command: Command,
    pub alias_suggestions: Vec<AliasSuggestion>,
}

pub fn get_suggestions_with_aliases(
    num: Option<usize>, 
    db: &mut Database, 
    alias_file_path: &str
) -> Vec<CommandWithAlias> {
    db.update_db();
    let commands = db.get_top_commands(num);
    
    let suggester = AliasSuggester::new(alias_file_path);
    
    commands.into_iter().map(|cmd| {
        let alias_suggestions = suggester.suggest_aliases(&cmd.command_text);
        CommandWithAlias {
            command: cmd.clone(),
            alias_suggestions,
        }
    }).collect()
}

