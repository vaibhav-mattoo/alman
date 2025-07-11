use std::fs::File;
use std::io::{BufRead, Write, BufReader};

// funcs to do:
// func to get vector < pair < command, alias > > from file
// func to add alias to file
// func to remove alias from file
// func to get list of aliases from file

pub fn get_aliases(file_path: &str) -> Vec<(String, String)> {
    // Try to open the file, create it if it doesn't exist
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            // File doesn't exist, create it
            match File::create(file_path) {
                Ok(_) => {
                    // Created new alias file
                    return Vec::new();
                }
                Err(_) => {
                    // Could not create alias file
                    return Vec::new();
                }
            }
        }
    };

    let reader = BufReader::new(file);
    let mut aliases = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => {
                continue;
            }
        };
        let line = line.trim();
        if line.starts_with("alias ") {
            let line_in = line[6..].trim();
            // now split by =
            if let Some(eq_index) = line_in.find('=') {
                let alias = line_in[..eq_index].trim().to_string();
                let mut command = line_in[(eq_index + 1)..].trim();
                if (command.starts_with('\'') && command.ends_with('\'')) ||
                   (command.starts_with('"') && command.ends_with('"')) {
                    command = &command[1..command.len()-1];
                }
                let command = command.to_string();
                aliases.push((alias, command));
            } else {
                // Invalid alias format
            }
        }
    }
    aliases
}

pub fn write_aliases(file_path: &str, aliases: Vec<(String, String)>) {
    let mut file = File::create(file_path).expect("Could not create alias file");
    for (alias, command) in aliases {
        let line = format!("alias {}='{}'\n", alias, command);
        file.write_all(line.as_bytes()).expect("Could not write to alias file");
    }
    file.flush().expect("Could not flush alias file");
}

pub fn add_alias_to_file(file_path: &str, alias: &str, command: &str) {
    let mut aliases = get_aliases (file_path);
    // Check if alias already exists, just the alias part
    if aliases.iter().any(|(a, _)| a == alias) {
        return;
    }
    // Add new alias
    aliases.push((alias.to_string(), command.to_string()));
    write_aliases(file_path, aliases);
}

pub fn remove_alias_from_file(file_path: &str, alias: &str) {
    let mut aliases = get_aliases(file_path);
    // Remove alias if it exists
    if let Some(pos) = aliases.iter().position(|(a, _)| a == alias) {
        aliases.remove(pos);
        write_aliases(file_path, aliases);
    }
}

pub fn get_aliases_from_multiple_files(file_paths: &[String]) -> Vec<(String, String)> {
    let mut all_aliases = Vec::new();
    
    for file_path in file_paths {
        let aliases = get_aliases(file_path);
        all_aliases.extend(aliases);
    }
    
    all_aliases
}

pub fn add_alias_to_multiple_files(file_paths: &[String], alias: &str, command: &str) {
    // Check if alias exists in any file
    let all_aliases = get_aliases_from_multiple_files(file_paths);
    if all_aliases.iter().any(|(a, _)| a == alias) {
        return;
    }
    
    // Add to the first file (primary alias file)
    if let Some(primary_file) = file_paths.first() {
        add_alias_to_file(primary_file, alias, command);
    }
}

pub fn remove_alias_from_multiple_files(file_paths: &[String], alias: &str) {
    let mut found = false;
    
    for file_path in file_paths {
        let mut aliases = get_aliases(file_path);
        if let Some(pos) = aliases.iter().position(|(a, _)| a == alias) {
            aliases.remove(pos);
            write_aliases(file_path, aliases);
            found = true;
            break; // Remove from first file where found
        }
    }
    
    if !found {
        return;
    }
}