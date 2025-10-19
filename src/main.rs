use file_deduplicator::{DuplicateFinder, file_generators};

use serde::Deserialize;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};

const CONFIG_FILE: &str = "conf.json";

#[derive(Debug, Deserialize)]
struct Config {
    target_dir: String,
}

fn read_config() -> Result<Config, Box<dyn Error>> {
    let config_path = Path::new(CONFIG_FILE);
    if !config_path.exists() {
        return Err(format!("Config file '{}' not found. Please make sure it exists in the same directory as the program.", CONFIG_FILE).into());
    }

    let config_data = fs::read_to_string(CONFIG_FILE)
        .map_err(|e| format!("Failed to read config file '{}': {}", CONFIG_FILE, e))?;

    let config: Config = serde_json::from_str(&config_data)
        .map_err(|e| format!("Invalid JSON in config file '{}': {}", CONFIG_FILE, e))?;

    Ok(config)
}

fn main() {
    run_program().expect("TODO: panic message");
}

fn run_program() -> Result<(), Box<dyn std::error::Error>> {
    // file_generators::generate_test_files()?;
    let config_data = read_config()?;
    println!("Config file loaded successfully");

    let input_dir = Path::new(&config_data.target_dir);
    println!("Target directory: {}", input_dir.display());

    if !input_dir.exists() {
        return Err(format!("Directory '{}' does not exist", input_dir.display()).into());
    }

    if !input_dir.is_dir() {
        return Err(format!("'{}' is not a directory", input_dir.display()).into());
    }

    let output_dir: PathBuf = env::current_dir()?.join("output");
    fs::create_dir_all(&output_dir)?;

    let duplicator =
        DuplicateFinder::new(input_dir.to_str().unwrap(), output_dir.to_str().unwrap())?;
    duplicator.copy_full_duplicates()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main();
        assert!(true);
    }
}
