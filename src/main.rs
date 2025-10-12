use file_deduplicator::DuplicateFinder;

fn main() {
    let duplicator = DuplicateFinder::new("./test_files", "output").unwrap();
    duplicator.move_full_duplicates().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main();
        assert!(true);
    }
}
