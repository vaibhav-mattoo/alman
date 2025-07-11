use crate::database::database_structs::{Database, DeletedCommands};

pub fn insert_command(command_str: String, db: &mut Database, deleted_commands: &DeletedCommands) {
    // now we should find every prefix of the command and insert that
    // for example, if git add . ; we should insert git, git add, git add .
    // should have single space between words in command, input may have multiple spaces
    let command_str = command_str.trim().to_string();
    if command_str.is_empty() {
        return; // Do not insert empty commands
    }
    let command_parts: Vec<&str> = command_str.split_whitespace().collect();
    if command_parts.is_empty() {
        return; // Do not insert commands with no words
    }
    
    // Skip commands that start with the current binary name
    let binary_name = std::env::args()
        .next()
        .and_then(|path| std::path::Path::new(&path).file_name().map(|f| f.to_os_string()))
        .and_then(|os_str| os_str.into_string().ok());
    if let Some(name) = binary_name {
        if command_parts[0] == name {
            return;
        }
    }
    
    // maintain a string called temp, which stores command so far and then we do a for loop
    let mut temp = String::new();
    for word in command_parts.iter() {
        if !temp.is_empty() {
            temp.push(' '); // Add a space before the next word
        }
        temp.push_str(word);
        let tempp = temp.clone();
        // Insert the current command prefix into the database
        db.add_command(tempp, deleted_commands);
    }
    db.add_command(command_str, deleted_commands);
}