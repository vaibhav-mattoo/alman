use std::fs;
use std::path::Path;
use serde_json;
use super::database_structs::{Database, DeletedCommands};

pub const DB_FILE: &str = "command_database.json";
pub const DELETED_COMMANDS_FILE: &str = "deleted_commands.json";
pub const CONFIG_FILE: &str = "config.json";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub alias_file_paths: Vec<String>,
}

pub fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    let json = serde_json::to_string_pretty(config)?;
    fs::write(config_path, json)?;
    Ok(())
}

pub fn load_config() -> Option<AppConfig> {
    let config_path = get_config_path();
    if !Path::new(&config_path).exists() {
        return None;
    }
    let content = fs::read_to_string(config_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn get_config_path() -> String {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home_dir.join(".alman").join(CONFIG_FILE).to_string_lossy().to_string()
}

pub fn save_database(db: &Database, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(db)?;
    fs::write(file_path, json)?;
    Ok(())
}

pub fn load_database(file_path: &str) -> Result<Database, Box<dyn std::error::Error>> {
    if !Path::new(file_path).exists() {
        // Return empty database if file doesn't exist
        return Ok(Database {
            command_list: std::collections::BTreeSet::new(),
            reverse_command_map: std::collections::HashMap::new(),
            total_num_commands: 0,
            total_score: 0,
        });
    }
    
    let content = fs::read_to_string(file_path)?;
    let db: Database = serde_json::from_str(&content)?;
    Ok(db)
}

pub fn save_deleted_commands(deleted_commands: &DeletedCommands, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(deleted_commands)?;
    fs::write(file_path, json)?;
    Ok(())
}

pub fn load_deleted_commands(file_path: &str) -> Result<DeletedCommands, Box<dyn std::error::Error>> {
    if !Path::new(file_path).exists() {
        // Return empty deleted commands if file doesn't exist
        return Ok(DeletedCommands {
            deleted_commands: std::collections::BTreeSet::new(),
        });
    }
    
    let content = fs::read_to_string(file_path)?;
    let deleted_commands: DeletedCommands = serde_json::from_str(&content)?;
    Ok(deleted_commands)
}

pub fn get_database_path() -> String {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home_dir.join(".alman").join(DB_FILE).to_string_lossy().to_string()
}

pub fn get_deleted_commands_path() -> String {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home_dir.join(".alman").join(DELETED_COMMANDS_FILE).to_string_lossy().to_string()
}

pub fn ensure_data_directory() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let data_dir = home_dir.join(".alman");
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    
    Ok(())
}

pub fn get_data_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let data_dir = home_dir.join(".alman");
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    
    Ok(data_dir)
}

pub fn get_default_alias_file_path() -> String {
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    home_dir.join(".alman").join("aliases").to_string_lossy().to_string()
} 