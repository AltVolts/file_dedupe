use crate::plugins::{
    IDeduplicatorPlugin, IntoResultFileInfo, IsUnique, PluginResult, ResultFileInfo,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct FullHashPlugin;

impl IDeduplicatorPlugin for FullHashPlugin {
    fn name(&self) -> &str {
        "full_hash"
    }

    fn exec<P: AsRef<Path>>(
        &self,
        file_list: &[P],
    ) -> Result<PluginResult, Box<dyn std::error::Error>> {
        let mut unique_files: Vec<ResultFileInfo> = Vec::new();
        let mut duplicate_files: Vec<Vec<ResultFileInfo>> = Vec::new();

        let mut hash_groups: HashMap<String, Vec<ResultFileInfo>> = HashMap::new();

        for file in file_list {
            if let Ok(hash) = compute_full_hash(file) {
                if let Some(group) = hash_groups.get_mut(&hash) {
                    if group.len() == 1 {
                        group[0].is_unique = IsUnique::Duplicate;
                    }
                    let is_unique = IsUnique::Duplicate;
                    let file_info =
                        file.into_result_file_info(hash, is_unique, self.name().to_string());
                    group.push(file_info);
                } else {
                    let is_unique = IsUnique::Unique;
                    let file_info = file.into_result_file_info(
                        hash.clone(),
                        is_unique,
                        self.name().to_string(),
                    );
                    let group = vec![file_info];
                    hash_groups.insert(hash, group);
                }
            }
        }

        for group in hash_groups.values() {
            if group.len() == 1 {
                unique_files.push(group[0].clone())
            } else {
                duplicate_files.push(group.clone())
            }
        }
        Ok(PluginResult {
            unique_files,
            duplicate_files,
        })
    }
}

pub fn compute_full_hash<P: AsRef<Path>>(file_path: P) -> std::io::Result<String> {
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



mod tests {
    use super::*;

    #[test]
    fn test_full_hash_plugin_name() {
        let full_hash_plugin = FullHashPlugin;
        let result = full_hash_plugin.name();
        assert!(result.starts_with("full_hash"));
    }

    #[test]
    fn test_full_hash_plugin_exec() {
        use crate::plugins::input_plugins::get_input_files;
        // unique - 10, full_duples - 9, partial_duples - 5, all - 24
        let files = get_input_files(&Path::new("./test_data"));
        let full_hash_plugin = FullHashPlugin;
        let result = full_hash_plugin.exec(&files).unwrap();
        assert_eq!(result.unique_files.len(), 10 + 5);
        assert_eq!(result.duplicate_files.len(), 3);
        assert_eq!(result.duplicate_files
                       .iter()
                       .map(|group| group.len())
                       .fold(0, |acc, e| acc + e), 9)
    }
}