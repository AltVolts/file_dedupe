use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use defaultdict::DefaultHashMap;


pub struct DuplicateFinder {
    input_dir: PathBuf,
    output_dir: PathBuf,
}


fn get_file_hash(file: &PathBuf) -> u64 {

    12
}

pub fn split_files_by_extensions(directory: &Path)
                             -> DefaultHashMap<String, Vec<PathBuf>>
{
    let mut files_by_ext: DefaultHashMap<String, Vec<PathBuf>> = DefaultHashMap::new();
    let dir_iter = WalkDir::new(directory);
    for entry in dir_iter.into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let path = entry.path().to_path_buf();
        let extension =path
            .extension()
            .unwrap_or(OsStr::new("unknown"))
            .to_string_lossy()
            .to_string();

        files_by_ext.get_mut(&extension).push(path);
    }
    files_by_ext
}

impl DuplicateFinder {
    fn new(input_path: &str, output_path: &str) -> Result<DuplicateFinder, String> {
        let input_dir = Self::validate_path(PathBuf::from(input_path))?;
        let output_dir = Self::validate_path(PathBuf::from(output_path))?;

        if Self::validate_dir_not_empty(&input_dir) {
            Ok(DuplicateFinder{input_dir, output_dir})
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
        WalkDir::new(path).into_iter().any(|e| e.unwrap().file_type().is_file())
    }



    // pub fn find_full_duplicates() -> Vec<PathBuf> {
    //     ()
    // }
    //
    // pub fn find_partial_duplicates() -> () {
    //     ()
    // }
}






#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile, tempfile_in};
    use std::fs;

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

        let finder = DuplicateFinder::new(
            input_temp.path().to_str().unwrap(),
            output_temp,
        );

        assert!(finder.is_err());

        let input_temp = "./alo";
        let output_temp = tempdir().unwrap();
        let finder = DuplicateFinder::new(
            input_temp,
            output_temp.path().to_str().unwrap(),
        );
        assert!(finder.is_err());

        let input_temp_file = NamedTempFile::new().unwrap();
        let output_temp_file = NamedTempFile::new().unwrap();

        let input_temp_path = input_temp_file.path().to_str().unwrap();
        let output_temp_path = output_temp_file.path().to_str().unwrap();

        let finder = DuplicateFinder::new(
            input_temp_path,
            output_temp_path,
        );
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
    fn test_find_full_duplicates() {
        ()
    }

    #[test]
    fn test_find_partial_duplicates() {
        ()
    }


}