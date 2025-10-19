use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkResult {
    pub method_name: String,
    pub total_files: usize,
    pub duplicates_found: usize,
    pub unique_files: usize,
    pub space_saved: u64,
    pub execution_time: Duration,
    pub memory_usage: usize,
    pub accuracy: f64,
    pub false_positives: usize,
    pub false_negatives: usize,
}

#[derive(Debug, Serialize)]
pub struct ComparisonReport {
    pub results: Vec<BenchmarkResult>,
    pub best_method: String,
    pub comparison_table: String,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub hash: Option<String>,
}
