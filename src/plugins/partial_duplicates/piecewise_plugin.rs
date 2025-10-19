use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use sha2::Sha256;
use crate::plugins::{IDeduplicatorPlugin, PluginResult, compute_md5};


pub struct PieceWisePlugin {
    min_chunk_size: usize,
    max_chunk_size: usize,
    mask_bits: u32,
}

impl PieceWisePlugin {
    pub fn new() -> Self {
        Self {
            min_chunk_size: 2048,    // 2KB minimum chunk size
            max_chunk_size: 65536,   // 64KB maximum chunk size
            mask_bits: 12,           // 12 bits for chunk boundary detection
        }
    }
    pub fn with_sizes(min_chunk_size: usize, max_chunk_size: usize, mask_bits: u32) -> Self {
        Self {
            min_chunk_size,
            max_chunk_size,
            mask_bits,
        }
    }
}

impl IDeduplicatorPlugin for PieceWisePlugin {
    fn name(&self) -> &str {
        "piece_wise"
    }

    fn exec<P: AsRef<Path>>(&self, file_list: &[P]) -> Result<PluginResult, Box<dyn Error>> {
        todo!()
    }
}


fn compute_piecewise_hash<P: AsRef<Path>>(file_path: P) -> std::io::Result<String> {
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

fn split_file_fixed_chunks(
    file_path: &str,
    chunk_size: usize
) -> Result<Vec<Vec<u8>>, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut chunks = Vec::new();
    let mut buffer = vec![0u8; chunk_size];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        chunks.push(buffer[..bytes_read].to_vec());
    }

    Ok(chunks)
}