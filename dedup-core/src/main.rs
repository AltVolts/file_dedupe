use std::{process};
use std::path::{Path};
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
    println!("Deduplicator run on directory [{:?}]", config_file.root_paths);
    println!("Config file loaded successfully");

    let scan_config = ScanConfig::build(config_file)?;

    // Создаем callback с поддержкой Send + Sync
    let callback: Option<Box<dyn Fn(ProgressUpdate) + Send + Sync>> = Some(Box::new(|update| {
        match update {
            ProgressUpdate::Scanning { current_path, files_scanned } => {
                if files_scanned % 100 == 0 {
                    println!("Найдено файлов: {}", files_scanned);
                }
            }
            ProgressUpdate::Finished(stats) => {
                println!("Завершено! Файлов: {}, Размер: {} MB, Время: {:?}",
                         stats.files_scanned,
                         stats.total_size / 1024 / 1024,
                         stats.elapsed());
            }
            _ => {}
        }
    }));

    let scanner = Scanner::new(scan_config, callback);


    let files = scanner.scan()?;

    let files = scanner.scan_parallel()?;
    Ok(())
}