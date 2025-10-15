use std::fs;
use rand::Rng;


fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";

    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn create_test_structure() -> std::io::Result<()> {
    let base_dirs = ["test_data", "test_data/subdir1", "test_data/subdir2"];

    for dir in &base_dirs {
        fs::create_dir_all(dir)?;
    }

    // Create duplicate files across different directories
    let duplicate_contents = vec![
        "This is duplicate content A".to_string(),
        "Duplicate content B with some data".to_string(),
        generate_random_string(500),
    ];

    for (i, content) in duplicate_contents.iter().enumerate() {
        // Same content in different directories
        let paths = [
            format!("test_data/duplicate_{}.txt", i),
            format!("test_data/subdir1/copy_duplicate_{}.txt", i),
            format!("test_data/subdir2/another_duplicate_{}.txt", i),
        ];

        for path in paths {
            fs::write(&path, content)?;
        }
    }

    // Create unique files
    for i in 0..10 {
        let unique_content = generate_random_string(100 + i * 50);
        let filename = format!("test_data/unique_{}.txt", i);
        fs::write(filename, unique_content)?;
    }

    // Create partially similar files
    let base_text = "This is a base text that will be partially duplicated. ";
    for i in 0..5 {
        let unique_part = generate_random_string(50);
        let content = format!("{}{}", base_text, unique_part);
        fs::write(format!("test_data/partial_dup_{}.txt", i), content)?;
    }

    Ok(())
}

pub fn generate_test_files() -> std::io::Result<()> {
    println!("Generating test file structure...");
    create_test_structure()?;
    println!("Test files created successfully!");
    Ok(())
}