use crate::benchmark::FileInfo;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

pub trait DeduplicationMethod: Send + Sync {
    fn name(&self) -> &str;
    fn find_duplicates(&self, files: &[FileInfo]) -> Vec<Vec<FileInfo>>;
}

pub struct SizeOnlyMethod;
impl DeduplicationMethod for SizeOnlyMethod {
    fn name(&self) -> &str {
        "size_only"
    }

    fn find_duplicates(&self, files: &[FileInfo]) -> Vec<Vec<FileInfo>> {
        let mut size_groups: HashMap<u64, Vec<FileInfo>> = HashMap::new();

        for file in files {
            size_groups
                .entry(file.size)
                .or_insert_with(Vec::new)
                .push(file.clone());
        }

        size_groups
            .into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }
}

pub struct PartialHashMethod;
impl DeduplicationMethod for PartialHashMethod {
    fn name(&self) -> &str {
        "partial_hash"
    }

    fn find_duplicates(&self, files: &[FileInfo]) -> Vec<Vec<FileInfo>> {
        let mut hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();

        for file in files {
            if let Ok(hash) = compute_partial_hash(&file.path, 1024) {
                hash_groups
                    .entry(hash)
                    .or_insert_with(Vec::new)
                    .push(file.clone());
            }
        }

        hash_groups
            .into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }
}

pub struct FullHashMethod;
impl DeduplicationMethod for FullHashMethod {
    fn name(&self) -> &str {
        "full_hash"
    }

    fn find_duplicates(&self, files: &[FileInfo]) -> Vec<Vec<FileInfo>> {
        let mut hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();

        for file in files {
            if let Ok(hash) = compute_full_hash(&file.path) {
                hash_groups
                    .entry(hash)
                    .or_insert_with(Vec::new)
                    .push(file.clone());
            }
        }

        hash_groups
            .into_values()
            .filter(|group| group.len() > 1)
            .collect()
    }
}

pub struct TwoPassMethod;
impl DeduplicationMethod for TwoPassMethod {
    fn name(&self) -> &str {
        "two_pass"
    }

    fn find_duplicates(&self, files: &[FileInfo]) -> Vec<Vec<FileInfo>> {
        // First pass: group by size
        let size_groups: HashMap<u64, Vec<FileInfo>> =
            files.iter().fold(HashMap::new(), |mut map, file| {
                map.entry(file.size)
                    .or_insert_with(Vec::new)
                    .push(file.clone());
                map
            });

        // Second pass: hash only files with same size
        let mut result = Vec::new();
        for group in size_groups.into_values().filter(|g| g.len() > 1) {
            let mut hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();

            for file in group {
                if let Ok(hash) = compute_partial_hash(&file.path, 4096) {
                    hash_groups.entry(hash).or_insert_with(Vec::new).push(file);
                }
            }

            result.extend(hash_groups.into_values().filter(|g| g.len() > 1));
        }

        result
    }
}

fn compute_partial_hash(file_path: &str, bytes: usize) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; bytes];
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn compute_full_hash(file_path: &str) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
