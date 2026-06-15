use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "config.json";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub alias_file_paths: Vec<String>,
}

pub fn save_config(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path();
    ensure_config_directory()?;
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
    let config_dir = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    });
    config_dir
        .join("alman")
        .join(CONFIG_FILE)
        .to_string_lossy()
        .to_string()
}

pub fn ensure_data_directory() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = get_data_directory()?;
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    Ok(())
}

pub fn ensure_config_directory() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_directory()?;
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    Ok(())
}

pub fn get_data_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data_home).join("alman")
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".local").join("share").join("alman")
    };
    Ok(data_dir)
}

pub fn get_config_directory() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config_home).join("alman")
    } else {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".config").join("alman")
    };
    Ok(config_dir)
}

pub fn get_default_alias_file_path() -> String {
    let config_dir = get_config_directory().unwrap_or_else(|_| {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home_dir.join(".config").join("alman")
    });
    config_dir.join("aliases").to_string_lossy().to_string()
}
