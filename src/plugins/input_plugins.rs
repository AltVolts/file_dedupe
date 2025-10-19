use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn get_input_files(input_dir: &Path) -> Vec<PathBuf> {
    let dir_iter = WalkDir::new(input_dir);
    dir_iter
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.into_path()).collect()
}
