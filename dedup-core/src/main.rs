use std::{env, process};
use std::path::{Path, PathBuf};
use glob::Pattern;
use dedup_core::config;
use dedup_core::models::ProgressUpdate;
use dedup_core::scanner::{ScanConfig, Scanner};

const CONFIG_FILE: &str = "/mnt/new_disk/Dev/Rust/file_deduplicator/conf.json";

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

    let config = ScanConfig {
        root_paths: vec![PathBuf::from(config_file.target_dir)],
        min_file_size: None,
        max_file_size: None,
        follow_symlinks: false,
        exclude_patterns: vec![
            Pattern::new("*.tmp").unwrap(),
            Pattern::new("**/.git/**").unwrap(),
        ],
        max_depth: Some(10),
        skip_hidden: true,
    };

    // Создаем callback с поддержкой Send + Sync
    let callback: Option<Box<dyn Fn(ProgressUpdate) + Send + Sync>> = Some(Box::new(|update| {
        match update {
            ProgressUpdate::Scanning { current_path, files_scanned } => {
                if files_scanned % 10 == 0 {
                    println!("Найдено файлов: {}", files_scanned);
                }
            }
            ProgressUpdate::Finished(stats) => {
                println!("Завершено! Файлов: {}, Размер: {} KB, Время: {:?}",
                         stats.files_scanned,
                         stats.total_size / 1024,
                         stats.elapsed());
            }
            _ => {}
        }
    }));

    let scanner = Scanner::new(config, callback);


    let files = scanner.scan()?;

    let files = scanner.scan_parallel()?;
    Ok(())
}