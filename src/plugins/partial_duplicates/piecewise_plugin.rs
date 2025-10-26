use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use crate::plugins::IDeduplicatorPlugin;
use anyhow::Result;
use crate::plugins::input_plugins::get_input_files;

const PIECE_WISE_NAME: &'static str = "piece_wise";
const CHUNK_SIZE: usize = 4096;
const NUM_CHUNKS: usize = 200;


struct PieceWisePlugin {
    block_size: u8,
}


impl PieceWisePlugin {
    pub fn new(block_size: u8) -> Self {
        Self { block_size }
    }
}


fn compute_sha(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash_result = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_result);
    hash_array
}


fn compute_md5(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}


fn read_and_hash_chunks(
    file: &mut File,
    start_chunk: usize,
    num_chunks: usize,
    total_chunks: usize,
) -> io::Result<Vec<[u8; 32]>> {
    let mut hashes = Vec::with_capacity(num_chunks);
    let mut buffer = vec![0u8; CHUNK_SIZE];
    for chunk_index in start_chunk..(start_chunk + num_chunks).min(total_chunks) {
        let offset = (chunk_index * CHUNK_SIZE) as u64;
        file.seek(SeekFrom::Start(offset))?;

        let bytes_read = file.read(&mut buffer)?;
        if bytes_read > 0 {
            let hash_result = compute_sha(&buffer);
            hashes.push(hash_result);
        }
    }
    Ok(hashes)
}

fn compute_file_hashes<P: AsRef<Path>>(file_path: P) -> io::Result<Vec<[u8; 32]>> {
    let mut file = File::open(file_path)?;
    let file_size = file.metadata()?.len();

    let total_chunks = (file_size as usize + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let mut hashes = Vec::with_capacity(total_chunks);

    hashes.extend(read_and_hash_chunks(&mut file, 0, NUM_CHUNKS, total_chunks)?);

    let start_chunk = total_chunks.saturating_sub(NUM_CHUNKS);
    hashes.extend(read_and_hash_chunks(&mut file, start_chunk, NUM_CHUNKS, total_chunks)?);
    Ok(hashes)
}

pub fn compute_file_chunk_hashes_hex<P: AsRef<Path>>(
    file_path: P,
) -> io::Result<Vec<String>> {
    let hashes = compute_file_hashes(file_path)?;
    Ok(hashes
        .into_iter()
        .map(|hash| hex::encode(hash))
        .collect())
}

fn compare_files(f1_hashes: &Vec<[u8; 32]>, f2_hashes: &Vec<[u8; 32]>) -> f64 {
    let total_hashes = f1_hashes.len().min(f2_hashes.len());
    let mut identical_hashes = 0;

    for index in 0..total_hashes {
        let hash1 = f1_hashes.get(index);
        let hash2 = f2_hashes.get(index);
        if hash1 == hash2 && hash1.is_some() {
           identical_hashes += 1;
        }
    }
    let similarity = if total_hashes > 0 {
        (identical_hashes as f64 / total_hashes as f64) * 100.0
    } else { 100.0 };
    similarity
}


fn compare_file_list<P: AsRef<Path>>(files: &Vec<P>) -> Vec<Vec<f64>> {
    let matrix_size = files.len();
    let mut result_matrix = vec![vec![0.0; matrix_size]; matrix_size];
    let file_list_hashes: Vec<Vec<[u8; 32]>> = files
        .iter()
        .map(|f| compute_file_hashes(f).unwrap())
        .collect();

    for i in 0..matrix_size {
        for j in 0..matrix_size {
            let similarity = compare_files(&file_list_hashes[i], &file_list_hashes[j]);
            result_matrix[i][j] = similarity;
        }
    }
    result_matrix
}


fn find_similar_files(input_dir: &Path, threshold: f64) -> Vec<(PathBuf, Vec<(PathBuf, f64)>)> {
    let files = get_input_files(input_dir);
    let result_matrix = compare_file_list(&files);

    let mut similar_files_list = Vec::new();

    for (idx, file) in files.iter().enumerate() {
        let similarities = result_matrix.get(idx).unwrap();
        let mut similar_files = Vec::new();

        for (other_idx, similarity) in similarities.iter().enumerate() {
            if *similarity >= threshold && idx != other_idx {
                similar_files.push((files[other_idx].clone(), *similarity));
            }
        }

        similar_files_list.push((file.clone(), similar_files));
    }


    for (file, similar_files) in &similar_files_list {
        if !similar_files.is_empty() {
            println!("File: {:?}", file);
            println!("Similar files:");
            for (similar_file, similarity) in similar_files {
                println!("  {:?} - {:.2}% similar", similar_file, similarity);
            }
            println!();
        }
    }

    similar_files_list
}




//
// impl IDeduplicatorPlugin for PieceWisePlugin {
//     fn name(&self) -> &str {
//         PIECE_WISE_NAME
//     }
//
//     fn exec<P: AsRef<Path>>(&self, file_list: &[P]) -> Result<PluginResult, Box<dyn Error>> {
//         let threshold = 0.9;
//         let mut unique_files: Vec<ResultFileInfo> = Vec::new();
//         let mut duplicate_files: Vec<Vec<ResultFileInfo>> = Vec::new();
//
//         // Вычисляем хеши для всех файлов
//         let files_hashes: Vec<(String, HashSet<String>)> = file_list
//             .iter()
//             .filter_map(|file_path| {
//                 let path_str = file_path.as_ref().to_string_lossy().to_string();
//                 match compute_piecewise_hash(file_path) {
//                     Ok(hashes) => Some((path_str, hashes)),
//                     Err(_) => None, // Пропускаем файлы с ошибками
//                 }
//             })
//             .collect();
//
//         // Группируем файлы по схожести
//         let mut processed = HashSet::new();
//
//         for i in 0..files_hashes.len() {
//             if processed.contains(&i) {
//                 continue;
//             }
//
//             let (current_path, current_hashes) = &files_hashes[i];
//             let mut duplicate_group = Vec::new();
//
//             // Создаем ResultFileInfo для текущего файла
//             let current_file_info = ResultFileInfo {
//                 path: current_path.clone(),
//                 size: std::fs::metadata(&current_path)?.len(),
//             };
//
//             duplicate_group.push(current_file_info);
//             processed.insert(i);
//
//             // Ищем дубликаты
//             for j in (i + 1)..files_hashes.len() {
//                 if processed.contains(&j) {
//                     continue;
//                 }
//
//                 let (other_path, other_hashes) = &files_hashes[j];
//                 let similarity = calculate_similarity(&current_hashes, &other_hashes);
//
//                 if similarity >= threshold {
//                     let other_file_info = ResultFileInfo {
//                         path: other_path.clone(),
//                         size: std::fs::metadata(other_path)?.len(),
//                     };
//                     duplicate_group.push(other_file_info);
//                     processed.insert(j);
//                 }
//             }
//
//             if duplicate_group.len() == 1 {
//                 unique_files.push(duplicate_group.remove(0));
//             } else {
//                 duplicate_files.push(duplicate_group);
//             }
//         }
//
//         Ok(PluginResult {
//             unique_files,
//             duplicate_files,
//         })
//     }
// }
//
//
// fn compute_piecewise_hash<P: AsRef<Path>>(file_path: P) -> std::io::Result<HashSet<String>> {
//     let chunks = split_file_fixed_chunks(&file_path, 1024)?;
//     let hashes = chunks
//         .iter()
//         .map(|chunk| compute_md5(chunk))
//         .collect();
//     Ok(hashes)
// }
//
// fn split_file_fixed_chunks<P: AsRef<Path>>(
//     file_path: &P,
//     chunk_size: usize
// ) -> Result<Vec<Vec<u8>>, std::io::Error> {
//     let mut file = File::open(file_path)?;
//     let mut chunks = Vec::new();
//     let mut buffer = vec![0u8; chunk_size];
//
//     loop {
//         let bytes_read = file.read(&mut buffer)?;
//         if bytes_read == 0 {
//             break;
//         }
//         chunks.push(buffer[..bytes_read].to_vec());
//     }
//     Ok(chunks)
// }