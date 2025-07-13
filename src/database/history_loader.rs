use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use super::database_structs::{Database, Command, DeletedCommands};

pub fn initialize_database_from_history(db: &mut Database, deleted_commands: &DeletedCommands) -> Result<(), Box<dyn std::error::Error>> {
    // Only initialize if database is empty
    if !db.command_list.is_empty() {
        return Ok(());
    }

    let history_file = get_history_file_path()?;
    if !Path::new(&history_file).exists() {
        return Ok(());
    }

    // Safety check: ensure we're not accidentally writing to the history file
    if history_file.contains("history") || history_file.contains("HIST") {
        // Double-check that we're only reading, not writing
        let metadata = fs::metadata(&history_file)?;
        if metadata.is_file() {
            // Read the file content safely, handling encoding errors
            let history_content = match fs::read_to_string(&history_file) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Warning: Could not read history file '{}': {}", history_file, e);
                    eprintln!("Attempting to read with lossy conversion...");
                    match fs::read(&history_file) {
                        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                        Err(e2) => {
                            eprintln!("Error: Could not read history file at all: {}", e2);
                            return Ok(());
                        }
                    }
                }
            };
            
            let commands = parse_history_file(&history_content);
            
            if commands.is_empty() {
                return Ok(());
            }

            // Calculate time intervals (2 minutes apart, going backwards from now)
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let interval_seconds = 120; // 2 minutes
            
            for (i, command) in commands.iter().enumerate() {
                let timestamp = now - (i as u64 * interval_seconds);
                insert_command_with_timestamp(command, timestamp, db, deleted_commands);
            }

            println!("Initialized database with {} commands from history", commands.len());
        }
    }
    
    Ok(())
}

fn get_history_file_path() -> Result<String, Box<dyn std::error::Error>> {
    // Try to get HISTFILE from environment
    if let Ok(histfile) = env::var("HISTFILE") {
        if !histfile.is_empty() {
            return Ok(histfile);
        }
    }

    // Fallback to shell-specific default history files in order of preference
    let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
    
    // Try common history file locations in order of preference
    let possible_paths = vec![
        home_dir.join(".zsh_history"),    // First priority
        home_dir.join(".bash_history"),   // Second priority
        home_dir.join(".history"),        // Third priority
        home_dir.join(".fish_history"),   // Fourth priority
    ];

    for path in possible_paths {
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    Err("No history file found".into())
}

fn parse_history_file(content: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    // Process lines from last to first (most recent first)
    for (line_num, line) in lines.iter().rev().enumerate() {
        // Skip lines that can't be processed due to encoding issues
        if !line.is_ascii() && !line.chars().all(|c| c.is_ascii() || c.is_whitespace()) {
            eprintln!("Warning: Skipping line {} due to invalid characters", line_num + 1);
            continue;
        }
        
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip lines that are likely not commands
        if trimmed.starts_with('#') || 
           trimmed.starts_with("HISTTIMEFORMAT") ||
           trimmed.starts_with("HISTSIZE") ||
           trimmed.starts_with("HISTFILESIZE") {
            continue;
        }

        // Extract command from history line
        let command = extract_command_from_history_line(trimmed);
        if !command.is_empty() && command.len() > 2 {
            commands.push(command);
        }
    }

    commands
}

fn extract_command_from_history_line(line: &str) -> String {
    // Handle different history file formats
    
    // Zsh history format: ": 1234567890:0;command"
    if line.starts_with(": ") {
        if let Some(semicolon_pos) = line.find(';') {
            let command_part = line[semicolon_pos + 1..].trim();
            // Only return if it's not empty and doesn't look like more metadata
            if !command_part.is_empty() && !command_part.starts_with(':') {
                // Additional safety check: ensure the command part is valid
                if command_part.chars().all(|c| c.is_ascii() || c.is_whitespace()) {
                    return command_part.to_string();
                }
            }
        }
        // If we can't parse it properly, return empty
        return String::new();
    }
    
    // Fish history format: "- cmd:command"
    if line.starts_with("- cmd:") {
        let command_part = line[6..].trim();
        if command_part.chars().all(|c| c.is_ascii() || c.is_whitespace()) {
            return command_part.to_string();
        }
        return String::new();
    }
    
    // Bash history format: just the command
    // But might have timestamps in some cases
    if line.starts_with('#') {
        // Skip timestamp lines
        return String::new();
    }
    
    // Default: assume it's just the command
    // Additional safety check for the default case
    if line.chars().all(|c| c.is_ascii() || c.is_whitespace()) {
        line.to_string()
    } else {
        String::new()
    }
}

fn insert_command_with_timestamp(command: &str, timestamp: u64, db: &mut Database, deleted_commands: &DeletedCommands) {
    if command.is_empty() || deleted_commands.deleted_commands.contains(command) {
        return;
    }

    // Skip commands that are too short or single-word
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.len() <= 1 || command.len() <= 5 {
        return;
    }

    // Skip commands that start with the current binary name
    let binary_name = std::env::args()
        .next()
        .and_then(|path| std::path::Path::new(&path).file_name().map(|f| f.to_os_string()))
        .and_then(|os_str| os_str.into_string().ok());
    if let Some(name) = binary_name {
        if parts[0] == name {
            return;
        }
    }

    // Create command with the specified timestamp
    let length: i16 = command.split_whitespace().map(|s| s.len()).sum::<usize>() as i16;
    let number_of_words: i8 = parts.len() as i8;
    let frequency: i32 = 1;

    let mut new_command = Command {
        score: 0, // Will be calculated below
        last_access_time: timestamp as i64,
        frequency,
        length,
        command_text: command.to_string(),
        number_of_words,
    };

    // Calculate score
    new_command.score = calculate_score(&new_command);

    // Insert command and all its prefixes
    let mut temp = String::new();
    for word in parts.iter() {
        if !temp.is_empty() {
            temp.push(' ');
        }
        temp.push_str(word);
        
        if temp.len() > 2 {
            let prefix_command = create_prefix_command(&temp, timestamp);
            db.add_command_with_existing(prefix_command);
        }
    }

    // Add the full command
    db.add_command_with_existing(new_command);
}

fn create_prefix_command(command_text: &str, timestamp: u64) -> Command {
    let length: i16 = command_text.split_whitespace().map(|s| s.len()).sum::<usize>() as i16;
    let number_of_words: i8 = command_text.split_whitespace().count() as i8;
    let frequency: i32 = 1;

    let mut command = Command {
        score: 0,
        last_access_time: timestamp as i64,
        frequency,
        length,
        command_text: command_text.to_string(),
        number_of_words,
    };

    command.score = calculate_score(&command);
    command
}

fn calculate_score(command: &Command) -> i32 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let time_difference: i64 = now - command.last_access_time;
    
    let mult: f64 = if time_difference <= 3600 {
        4.0
    } else if time_difference <= 86400 {
        2.0
    } else if time_difference <= 604800 {
        0.5
    } else {
        0.25
    };
    
    let length = command.length as f64;
    let frequency = command.frequency as f64;

    (mult * length.powf(3.0 / 5.0) * frequency) as i32
} 