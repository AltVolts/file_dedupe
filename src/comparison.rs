use crate::benchmark::{BenchmarkResult, ComparisonReport, FileInfo};
use crate::methods::DeduplicationMethod;
use std::collections::HashSet;
use std::time::Instant;
use sysinfo::{Process, System};

pub struct MethodComparator {
    methods: Vec<Box<dyn DeduplicationMethod>>,
    ground_truth: Option<Vec<Vec<FileInfo>>>,
}

impl MethodComparator {
    pub fn new() -> Self {
        Self {
            methods: Vec::new(),
            ground_truth: None,
        }
    }

    pub fn add_method<M: DeduplicationMethod + 'static>(&mut self, method: M) {
        self.methods.push(Box::new(method));
    }

    pub fn set_ground_truth(&mut self, truth: Vec<Vec<FileInfo>>) {
        self.ground_truth = Some(truth);
    }

    pub fn compare_methods(&self, test_files: &[FileInfo]) -> ComparisonReport {
        let mut results = Vec::new();

        for method in &self.methods {
            println!("Testing method: {}", method.name());

            let start_time = Instant::now();
            let memory_before = self.get_memory_usage();

            let duplicates = method.find_duplicates(test_files);

            let duration = start_time.elapsed();
            let memory_after = self.get_memory_usage();

            let result = self.analyze_result(
                method.name(),
                &duplicates,
                test_files,
                duration,
                memory_after.saturating_sub(memory_before),
            );

            results.push(result);
        }

        let best_method = self.determine_best_method(&results);
        let comparison_table = self.generate_comparison_table(&results);

        ComparisonReport {
            results,
            best_method,
            comparison_table,
        }
    }

    fn analyze_result(
        &self,
        method_name: &str,
        duplicates: &[Vec<FileInfo>],
        all_files: &[FileInfo],
        duration: std::time::Duration,
        memory_used: usize,
    ) -> BenchmarkResult {
        let total_duplicates: usize = duplicates.iter().map(|group| group.len() - 1).sum();
        let unique_files =
            all_files.len() - duplicates.iter().map(|g| g.len()).sum::<usize>() + duplicates.len();
        let space_saved: u64 = duplicates
            .iter()
            .map(|group| group[0].size * (group.len() - 1) as u64)
            .sum();

        let (accuracy, false_positives, false_negatives) =
            self.calculate_accuracy(method_name, duplicates, all_files);

        BenchmarkResult {
            method_name: method_name.to_string(),
            total_files: all_files.len(),
            duplicates_found: total_duplicates,
            unique_files,
            space_saved,
            execution_time: duration,
            memory_usage: memory_used,
            accuracy,
            false_positives,
            false_negatives,
        }
    }

    fn calculate_accuracy(
        &self,
        method_name: &str,
        duplicates: &[Vec<FileInfo>],
        all_files: &[FileInfo],
    ) -> (f64, usize, usize) {
        // For simplicity, use full hash method as ground truth if not provided
        if self.ground_truth.is_none() {
            let full_hash_method = crate::methods::FullHashMethod;
            let truth = full_hash_method.find_duplicates(all_files);
            return self.compare_with_ground_truth(duplicates, &truth, all_files);
        }

        self.compare_with_ground_truth(duplicates, self.ground_truth.as_ref().unwrap(), all_files)
    }

    fn compare_with_ground_truth(
        &self,
        detected: &[Vec<FileInfo>],
        truth: &[Vec<FileInfo>],
        all_files: &[FileInfo],
    ) -> (f64, usize, usize) {
        // Convert to sets for comparison
        let detected_set: HashSet<Vec<String>> = detected
            .iter()
            .map(|group| group.iter().map(|f| f.path.clone()).collect())
            .collect();

        let truth_set: HashSet<Vec<String>> = truth
            .iter()
            .map(|group| group.iter().map(|f| f.path.clone()).collect())
            .collect();

        let true_positives = detected_set.intersection(&truth_set).count();
        let false_positives = detected_set.difference(&truth_set).count();
        let false_negatives = truth_set.difference(&detected_set).count();

        let accuracy = if true_positives + false_positives + false_negatives > 0 {
            true_positives as f64 / (true_positives + false_positives + false_negatives) as f64
        } else {
            1.0
        };

        (accuracy, false_positives, false_negatives)
    }

    fn determine_best_method(&self, results: &[BenchmarkResult]) -> String {
        results
            .iter()
            .max_by(|a, b| {
                // Weighted scoring: 40% accuracy, 30% speed, 20% memory, 10% space saved
                let score_a = a.accuracy * 0.4
                    + (1.0 / (a.execution_time.as_secs_f64() + 0.001)) * 0.3
                    + (1.0 / (a.memory_usage as f64 + 1.0)) * 0.2
                    + (a.space_saved as f64 / 1_000_000.0) * 0.1;

                let score_b = b.accuracy * 0.4
                    + (1.0 / (b.execution_time.as_secs_f64() + 0.001)) * 0.3
                    + (1.0 / (b.memory_usage as f64 + 1.0)) * 0.2
                    + (b.space_saved as f64 / 1_000_000.0) * 0.1;

                score_a.partial_cmp(&score_b).unwrap()
            })
            .map(|r| r.method_name.clone())
            .unwrap_or_else(|| "none".to_string())
    }

    fn generate_comparison_table(&self, results: &[BenchmarkResult]) -> String {
        let mut table = String::new();

        table.push_str(&format!(
            "{:<15} {:<10} {:<8} {:<12} {:<10} {:<8} {:<10} {:<6}\n",
            "Method", "Time(ms)", "Memory", "Accuracy", "Duplicates", "Saved(MB)", "FP", "FN"
        ));
        table.push_str(&format!("{:-<80}\n", ""));

        for result in results {
            table.push_str(&format!(
                "{:<15} {:<10.2} {:<8} {:<12.4} {:<10} {:<8.2} {:<10} {:<6}\n",
                result.method_name,
                result.execution_time.as_millis(),
                result.memory_usage / 1024, // KB
                result.accuracy,
                result.duplicates_found,
                result.space_saved as f64 / 1_000_000.0,
                result.false_positives,
                result.false_negatives,
            ));
        }

        table
    }

    fn get_memory_usage(&self) -> usize {
        let mut system = System::new();
        system.refresh_memory();
        system.used_memory() as usize
    }
}
