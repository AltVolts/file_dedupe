use std::path::{Path, PathBuf};
use walkdir::{WalkDir, DirEntry};
use rayon::prelude::*;
pub use crate::config::ScanConfig;
use crate::models::{FileMetadata, ProgressUpdate, ScanStats};

pub struct Scanner {
    config: ScanConfig,
    progress_callback: Option<Box<dyn Fn(ProgressUpdate) + Send + Sync + 'static>>,
}

impl Scanner {
    pub fn new(config: ScanConfig, progress_callback: Option<Box<dyn Fn(ProgressUpdate) + Send + Sync + 'static>>) -> Self {
        Scanner { config, progress_callback }
    }

    pub fn scan(&self) -> Result<Vec<FileMetadata>, Box<dyn std::error::Error>> {
        let mut entries = Vec::new();
        let mut stats = ScanStats::new();

        for path in &self.config.root_paths {
            let mut files = self.walk_directory(path, &mut stats).collect::<Vec<_>>();
            entries.append(&mut files);
        }

        if let Some(callback) = &self.progress_callback {
            callback(ProgressUpdate::Finished(stats));
        }

        Ok(entries)
    }

    fn walk_directory<'a>(&'a self, root: &Path, stats: &'a mut ScanStats) -> impl Iterator<Item = FileMetadata> + 'a {
        let walker = WalkDir::new(root)
            .max_depth(self.config.max_depth.unwrap_or(usize::MAX))
            .follow_links(self.config.follow_symlinks)
            .into_iter()
            .filter_entry(|e| !self.should_skip(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        walker.filter_map(move |entry| {
            stats.files_scanned += 1;

            if let Some(callback) = &self.progress_callback {
                callback(ProgressUpdate::Scanning {
                    current_path: entry.path().to_path_buf(),
                    files_scanned: stats.files_scanned,
                });
            }

            self.create_metadata(entry, stats)
        })
    }

    fn should_skip(&self, entry: &DirEntry) -> bool {
        let path = entry.path();

        if self.config.exclude_patterns.iter().any(|pattern| pattern.matches_path(path)) {
            return true;
        }

        if let Some(name) = path.file_name() {
            if name.to_string_lossy().starts_with('.') {
                return true;
            }
        }

        false
    }

    fn create_metadata(&self, entry: DirEntry, stats: &mut ScanStats) -> Option<FileMetadata> {
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => return None,
        };

        let size = metadata.len();

        if let Some(min_size) = self.config.min_file_size {
            if size < min_size {
                return None;
            }
        }

        if let Some(max_size) = self.config.max_file_size {
            if size > max_size {
                return None;
            }
        }

        stats.total_size += size;

        Some(FileMetadata {
            path: entry.path().to_path_buf(),
            size,
            hash: None,
            modified: metadata.modified().ok(),
            created: metadata.created().ok(),
        })
    }

    pub fn scan_parallel(&self) -> Result<Vec<FileMetadata>, Box<dyn std::error::Error>> {
        let stats = std::sync::Arc::new(std::sync::Mutex::new(ScanStats::new()));

        let all_paths: Vec<PathBuf> = self.config.root_paths
            .iter()
            .flat_map(|path| {
                WalkDir::new(path)
                    .max_depth(self.config.max_depth.unwrap_or(usize::MAX))
                    .follow_links(self.config.follow_symlinks)
                    .into_iter()
                    .filter_entry(|e| !self.should_skip(e))
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.path().to_path_buf())
            })
            .collect();

        let results: Vec<FileMetadata> = all_paths
            .into_par_iter()
            .filter_map(|path| {
                let metadata = match std::fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => return None,
                };

                let size = metadata.len();

                if let Some(min_size) = self.config.min_file_size {
                    if size < min_size { return None; }
                }

                if let Some(max_size) = self.config.max_file_size {
                    if size > max_size { return None; }
                }

                // Update stats
                {
                    let mut stats = stats.lock().unwrap();
                    stats.files_scanned += 1;
                    stats.total_size += size;

                    // Call progress callback if exists
                    if let Some(callback) = &self.progress_callback {
                        callback(ProgressUpdate::Scanning {
                            current_path: path.clone(),
                            files_scanned: stats.files_scanned,
                        });
                    }
                }

                Some(FileMetadata {
                    path,
                    size,
                    hash: None,
                    modified: metadata.modified().ok(),
                    created: metadata.created().ok(),
                })
            })
            .collect();

        // Send final update
        if let Some(callback) = &self.progress_callback {
            let final_stats = stats.lock().unwrap().clone();
            callback(ProgressUpdate::Finished(final_stats));
        }

        Ok(results)
    }
}