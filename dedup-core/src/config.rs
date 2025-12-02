use std::{env, fs};
use std::path::Path;
use serde::Deserialize;



#[derive(Debug, Deserialize)]
pub struct Config {
    pub target_dir: String,
}

impl Config {
    pub fn build(path: &str) -> Self {
        Config{target_dir: path.to_string()}
    }
}

pub fn read_config_file(config_path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    if !config_path.exists() {
        return Err(format!("Config file '{}' not found. Please make sure it exists in the same directory as the program.", config_path.display()).into());
    }

    let config_data = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file '{}': {}", config_path.display(), e))?;

    let config: Config = serde_json::from_str(&config_data)
        .map_err(|e| format!("Invalid JSON in config file '{}': {}", config_path.display(), e))?;

    Ok(config)
}


pub fn read_config_cli() -> Result<Config, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Not enough arguments".into());
    }
    let target_dir = &args[1];
    Ok(Config::build(target_dir))
}