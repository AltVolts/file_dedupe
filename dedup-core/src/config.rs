use std::{env, fs};
use std::path::{Path, PathBuf};
use glob::Pattern;
use serde::Deserialize;



#[derive(Debug, Deserialize)]
pub struct Config {
    pub root_paths: Vec<PathBuf>,
    pub min_file_size: Option<u64>,
    pub max_file_size: Option<u64>,
    pub follow_symlinks: bool,
    pub exclude_patterns: Vec<String>,
    pub max_depth: Option<usize>,
    pub skip_hidden: bool,
}

#[derive(Debug)]
pub struct ScanConfig {
    pub root_paths: Vec<PathBuf>,
    pub min_file_size: Option<u64>,
    pub max_file_size: Option<u64>,
    pub follow_symlinks: bool,
    pub exclude_patterns: Vec<Pattern>,
    pub max_depth: Option<usize>,
    pub skip_hidden: bool,
}

impl ScanConfig {
    pub fn build(config: Config) -> Result<Self, glob::PatternError> {
        let exclude_patterns: Result<Vec<Pattern>, glob::PatternError> = config.exclude_patterns
            .into_iter()
            .map(|s| Pattern::new(&s))
            .collect();

        Ok(ScanConfig {
            root_paths: config.root_paths,
            min_file_size: config.min_file_size,
            max_file_size: config.max_file_size,
            follow_symlinks: config.follow_symlinks,
            exclude_patterns: exclude_patterns?,
            max_depth: config.max_depth,
            skip_hidden: config.skip_hidden,
        })
    }

    pub fn from_toml(toml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = toml::from_str(toml_str)?;
        ScanConfig::build(config).map_err(|e| e.into())
    }

    pub fn from_json(json_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = serde_json::from_str(json_str)?;
        ScanConfig::build(config).map_err(|e| e.into())
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