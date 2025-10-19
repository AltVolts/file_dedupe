use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use super::super::{IDeduplicatorPlugin, PluginResult};
use std::path::Path;
use sha2::{Digest, Sha256};
use crate::benchmark::FileInfo;
use crate::plugins::ResultFileInfo;

pub struct FullHashPlugin;

impl IDeduplicatorPlugin for FullHashPlugin {
    fn name(&self) -> &str {
        "full_hash"
    }

    fn exec<P: AsRef<Path>>(
        &self,
        file_list: &[P],
    ) -> Result<PluginResult, Box<dyn std::error::Error>> {
        let mut unique_files: Vec<Result<ResultFileInfo, String>> = Vec::new();
        let mut duplicates_files: Vec<Vec<Result<ResultFileInfo, String>>> = Vec::new();

        let mut hash_groups: HashMap<String, Vec<FileInfo>> = HashMap::new();

        for file in file_list {
            if let Ok(hash) = compute_full_hash(file) {
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


fn compute_full_hash<P: AsRef<Path>>(file_path: P) -> std::io::Result<String> {
    let file = File::open(&file_path)?;
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
