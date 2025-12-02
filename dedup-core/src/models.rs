use std::path::{PathBuf};
use std::time::SystemTime;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub hash: Option<FileHash>,
    pub inode: Option<u64>,
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