mod plugins;

// use plugins::{
//     Plugin,
//     PluginManager,
//     ScanPlugin,
//     ExtensionPlugin,
//     FullDuplicatePlugin,
//     PartialDuplicatePlugin,
// };

use anyhow::Result;
use defaultdict::DefaultHashMap;
use ring::digest::{Context, Digest, SHA256};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct DuplicateFinder {
    input_dir: PathBuf,
    output_dir: PathBuf,
}

fn get_file_sha256_digest<P: AsRef<Path>>(path: P) -> Result<[u8; 32]> {
    let input = File::open(path)?;
    let mut reader = BufReader::new(input);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    while let Ok(count) = reader.read(&mut buffer) {
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    let digest = context.finish();
    let mut result = [0u8; 32];
    result.copy_from_slice(digest.as_ref());

    Ok(result)
}

fn group_by_digest(file_list: &Vec<PathBuf>) -> DefaultHashMap<[u8; 32], Vec<PathBuf>> {
    let mut files_by_ext: DefaultHashMap<[u8; 32], Vec<PathBuf>> = DefaultHashMap::new();
    for filepath in file_list {
        if let Ok(digest) = get_file_sha256_digest(filepath) {
            files_by_ext.get_mut(&digest).push(filepath.clone());
        }
    }
    files_by_ext
}

pub fn split_files_by_extensions(directory: &Path) -> DefaultHashMap<String, Vec<PathBuf>> {
    let mut files_by_ext: DefaultHashMap<String, Vec<PathBuf>> = DefaultHashMap::new();
    let dir_iter = WalkDir::new(directory);
    for entry in dir_iter
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let path = entry.path().to_path_buf();
        let extension = path
            .extension()
            .unwrap_or(OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        files_by_ext.get_mut(&extension).push(path);
    }
    files_by_ext
}

impl DuplicateFinder {
    pub fn new(input_path: &str, output_path: &str) -> Result<DuplicateFinder, String> {
        let input_dir = Self::validate_path(PathBuf::from(input_path))?;
        let output_dir = Self::validate_path(PathBuf::from(output_path))?;

        if Self::validate_dir_not_empty(&input_dir) {
            Ok(DuplicateFinder {
                input_dir,
                output_dir,
            })
        } else {
            Err(format!("{} is empty", input_path))
        }
    }

    fn validate_path(path: PathBuf) -> Result<PathBuf, String> {
        if !path.exists() {
            return Err(format!("{:?} does not exist", path));
        }
        if !path.is_dir() {
            return Err(format!("{:?} is not a directory", path));
        }
        Ok(path)
    }

    fn validate_dir_not_empty(path: &Path) -> bool {
        WalkDir::new(path)
            .into_iter()
            .any(|e| e.unwrap().file_type().is_file())
    }

    pub fn find_full_duplicates(&self) -> Vec<Vec<PathBuf>> {
        let mut result = vec![];
        let files_map = split_files_by_extensions(&self.input_dir);
        for files_vec in files_map.values() {
            let new_duplicates = group_by_digest(files_vec);
            result.extend(new_duplicates.values().cloned());
        }
        result
    }

    pub fn move_full_duplicates(&self) -> Result<()> {
        let duplicate_files = self.find_full_duplicates();
        for (idx, files) in duplicate_files.iter().enumerate() {
            let new_path_group = self.output_dir.join(format!("{idx}"));
            fs::create_dir_all(&new_path_group)?;
            for (count, file) in files.iter().enumerate() {
                let original_filename = file
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown_file");

                let new_filename = if count == 0 {
                    format!("original_{}", original_filename)
                } else {
                    format!("[{}]_{}", count, original_filename)
                };

                let new_filepath = new_path_group.join(new_filename);
                fs::copy(file, &new_filepath)?;
            }
        }
        Ok(())
    }

    pub fn find_duplicates_with_info(&self) -> Vec<(String, Vec<PathBuf>)> {
        let mut result = Vec::new();
        let files_map = split_files_by_extensions(&self.input_dir);

        for (extension, files_vec) in files_map {
            let duplicates_by_digest = group_by_digest(&files_vec);

            for (digest, file_group) in duplicates_by_digest {
                if file_group.len() > 1 {
                    let digest_hex = hex::encode(digest);
                    result.push((format!("{}_{}", extension, digest_hex), file_group));
                }
            }
        }
        result
    }

    // pub fn find_partial_duplicates() -> () {
    //     ()
    // }
}

pub mod config {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DuplicateConfig {
        pub enabled_plugins: Vec<String>,
        pub case_sensitive: bool,
        pub min_length: usize,
    }

    impl Default for DuplicateConfig {
        fn default() -> Self {
            Self {
                enabled_plugins: vec!["uppercase".to_string(), "reverse".to_string()],
                case_sensitive: true,
                min_length: 1,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs;
    use tempfile::{NamedTempFile, tempdir, tempfile, tempfile_in};

    fn setup_test_environment() -> (tempfile::TempDir, tempfile::TempDir) {
        let input_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();

        // Create some test files
        fs::write(input_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(input_dir.path().join("file2.txt"), "content2").unwrap();

        (input_dir, output_dir)
    }

    #[test]
    fn test_duplicate_finder_creation() {
        let (input_dir, output_dir) = setup_test_environment();
        let finder = DuplicateFinder::new(
            input_dir.path().to_str().unwrap(),
            output_dir.path().to_str().unwrap(),
        );

        assert!(finder.is_ok());
    }

    #[test]
    fn test_failed_finder_creation() {
        let input_temp = tempdir().unwrap();
        let output_temp = "./alo";

        let finder = DuplicateFinder::new(input_temp.path().to_str().unwrap(), output_temp);

        assert!(finder.is_err());

        let input_temp = "./alo";
        let output_temp = tempdir().unwrap();
        let finder = DuplicateFinder::new(input_temp, output_temp.path().to_str().unwrap());
        assert!(finder.is_err());

        let input_temp_file = NamedTempFile::new().unwrap();
        let output_temp_file = NamedTempFile::new().unwrap();

        let input_temp_path = input_temp_file.path().to_str().unwrap();
        let output_temp_path = output_temp_file.path().to_str().unwrap();

        let finder = DuplicateFinder::new(input_temp_path, output_temp_path);
        assert!(finder.is_err());
    }

    #[test]
    fn test_empty_dir() {
        let input_temp = tempdir().unwrap();
        let output_temp = tempdir().unwrap();

        let finder = DuplicateFinder::new(
            input_temp.path().to_str().unwrap(),
            output_temp.path().to_str().unwrap(),
        );
        assert!(finder.is_err());
    }

    #[test]
    fn find_nested_files() {
        let input_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        let nested_dir = input_dir.path().join("alo");
        fs::create_dir(&nested_dir).unwrap();
        fs::write(nested_dir.join("file1.txt"), "content1").unwrap();
        let finder = DuplicateFinder::new(
            input_dir.path().to_str().unwrap(),
            output_dir.path().to_str().unwrap(),
        );
        assert!(finder.is_ok());
    }

    #[test]
    fn test_split_files_by_extensions() {
        let input_dir = tempdir().unwrap();

        // Create some test files
        fs::write(input_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(input_dir.path().join("file2.txt"), "content1").unwrap();
        fs::write(input_dir.path().join("file3.docx"), "content2").unwrap();
        fs::write(input_dir.path().join("file4.eml"), "content2").unwrap();

        let result = split_files_by_extensions(input_dir.path());
        assert_eq!(result.len(), 3);

        assert_eq!(result.get("txt").len(), 2);
        assert_eq!(result.get("docx").len(), 1);
        assert_eq!(result.get("eml").len(), 1);

        assert!(result.get("docx")[0].ends_with("file3.docx"));
        assert!(result.get("eml")[0].ends_with("file4.eml"));
    }

    #[test]
    fn test_get_file_sha256_digest() {
        use std::io::{Read, Seek, SeekFrom, Write};

        let mut file_1 = NamedTempFile::new().unwrap();
        writeln!(file_1, "123").unwrap();
        file_1.flush().unwrap();
        let digest_1 = get_file_sha256_digest(file_1.path()).unwrap();

        let mut file_2 = NamedTempFile::new().unwrap();
        writeln!(file_2, "123").unwrap();
        file_2.flush().unwrap();
        let digest_2 = get_file_sha256_digest(file_2.path()).unwrap();

        assert_eq!(digest_1, digest_2);

        let mut file_3 = NamedTempFile::new().unwrap();
        writeln!(file_3, "321").unwrap();
        file_3.flush().unwrap();
        let digest_3 = get_file_sha256_digest(file_3.path()).unwrap();
        assert_ne!(digest_2, digest_3);
    }

    fn create_test_files(dir: &Path, files: &[(&str, &[u8])]) -> Result<(), std::io::Error> {
        use std::io::Write;

        for (filename, content) in files {
            let file_path = dir.join(filename);
            let mut file = File::create(file_path)?;
            file.write_all(content)?;
        }
        Ok(())
    }

    // Helper to compare results regardless of order
    fn normalize_duplicates_result(mut result: Vec<Vec<PathBuf>>) -> Vec<Vec<PathBuf>> {
        for group in &mut result {
            group.sort();
        }
        result.sort_by(|a, b| a[0].cmp(&b[0]));
        result
    }

    #[test]
    fn test_find_full_duplicates_basic() {
        use tempfile::TempDir;

        let temp_dir = tempdir().unwrap();
        let input_dir = temp_dir.path();
        let nested_dir = input_dir.join("subdir");
        fs::create_dir(&nested_dir).unwrap();

        // Create files with duplicates
        let test_files: &[(&str, &[u8])] = &[
            ("file1.txt", b"identical content".as_slice()),
            ("file2.txt", b"identical content".as_slice()),
            ("file3.txt", b"different content".as_slice()),
            ("subdir/file4.txt", b"identical content".as_slice()),
        ];

        create_test_files(input_dir, &test_files).unwrap();

        let finder =
            DuplicateFinder::new(input_dir.to_str().unwrap(), input_dir.to_str().unwrap()).unwrap();

        let duplicates = finder.find_full_duplicates();
        let normalized = normalize_duplicates_result(duplicates);
        println!("{:?}", normalized);
        assert_eq!(normalized.len(), 2);
        assert_eq!(normalized[0].len(), 3);
    }

    // #[test]
    // fn test_find_partial_duplicates() {
    //     ()
    // }
}
