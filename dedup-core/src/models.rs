use std::path::{PathBuf};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub hash: Option<FileHash>,
    pub modified: Option<SystemTime>,
    pub created: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileHash {
    pub algorithm: HashAlgorithm,
    pub value: String,
    pub partial: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum HashAlgorithm {
    Blake3,
    SHA256,
    XXH3,
    MD5,
}

#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub files: Vec<FileMetadata>,
    pub total_size: u64,
    pub hash: FileHash,
}

#[derive(Debug, Clone)]
pub struct ScanStats {
    pub files_scanned: u64,
    pub total_size: u64,
    pub start_time: std::time::Instant,
}

impl ScanStats {
    pub fn new() -> Self {
        Self {
            files_scanned: 0,
            total_size: 0,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

#[derive(Debug, Clone)]
pub enum ProgressUpdate {
    Scanning {
        current_path: PathBuf,
        files_scanned: u64,
    },
    Processing {
        current_file: PathBuf,
        files_processed: u64,
        total_files: u64,
    },
    Hashing {
        current_file: PathBuf,
        bytes_hashed: u64,
        total_bytes: u64,
    },
    Finished(ScanStats),
}