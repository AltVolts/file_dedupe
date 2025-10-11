use std::path::{Path, PathBuf};


struct DuplicateFinder {
    input_dir: PathBuf,
    output_dir: PathBuf,
}


impl DuplicateFinder {
    fn new(input_path: &str, output_path: &str) -> Result<DuplicateFinder, String> {
        let input_dir = Self::validate_path(PathBuf::from(input_path))?;
        let output_dir = Self::validate_path(PathBuf::from(output_path))?;

        Ok(DuplicateFinder{input_dir, output_dir})
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

    pub fn find_duplicates(dir: &Path) -> () {
        ()
    }
}






#[cfg(test)]
mod tests {
    #[test]
    fn test_create_duplicate_finder() {

    }


    #[test]
    fn test_find_duplicates_1() {
        ()
    }
}