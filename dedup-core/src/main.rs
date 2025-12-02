use std::{env, process};
use std::path::Path;
use dedup_core::config;

const CONFIG_FILE: &str = "../conf.json";

fn main() {
    if let Err(e) = run_program() {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

fn run_program() -> Result<(), Box<dyn std::error::Error>> {
    let config_file_path = Path::new(CONFIG_FILE);
    println!("Using config file '{}'", config_file_path.display());
    let config_file = config::read_config_file(config_file_path)?;
    println!("Deduplicator run on directory [{}]", config_file.target_dir);
    println!("Config file loaded successfully");
    //
    // let input_dir = Path::new(&config_data.target_dir);
    // println!("Target directory: {}", input_dir.display());
    //
    // if !input_dir.exists() {
    //     return Err(format!("Directory '{}' does not exist", input_dir.display()).into());
    // }
    //
    // if !input_dir.is_dir() {
    //     return Err(format!("'{}' is not a directory", input_dir.display()).into());
    // }
    //
    // let output_dir: PathBuf = env::current_dir()?.join("output");
    // fs::create_dir_all(&output_dir)?;
    //
    // let duplicator =
    //     DuplicateFinder::new(input_dir.to_str().unwrap(), output_dir.to_str().unwrap())?;
    // duplicator.copy_full_duplicates()?;

    Ok(())
}