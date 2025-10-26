use std::fs;
use std::path::Path;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

mod full_duplicates;
mod partial_duplicates;
mod input_plugins;

#[derive(Debug)]
struct DuplicateGroup {
    group_number: usize,
    hash_list: Vec<String>,
}

#[derive(Debug, Clone)]
enum IsUnique {
    Unique,
    Duplicate,
}

#[derive(Debug, Clone)]
struct ResultFileInfo {
    path: String,
    name: String,
    size: u64,
    hash: String,
    is_unique: IsUnique,
    plugin_type: String,
}

trait IntoResultFileInfo {
    fn into_result_file_info(
        self,
        hash: String,
        is_unique: IsUnique,
        plugin_type: String,
    ) -> ResultFileInfo;
}

impl<P> IntoResultFileInfo for P
where
    P: AsRef<Path>,
{
    fn into_result_file_info(
        self,
        hash: String,
        is_unique: IsUnique,
        plugin_type: String,
    ) -> ResultFileInfo {
        let path_ref = self.as_ref();

        let size = fs::metadata(path_ref).map(|m| m.len()).unwrap_or(0);

        let name = path_ref
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let path = path_ref
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "invalid_path".to_string());

        ResultFileInfo {
            path,
            name,
            size,
            hash,
            is_unique,
            plugin_type,
        }
    }
}

#[derive(Debug)]
struct PluginResult {
    unique_files: Vec<ResultFileInfo>,
    duplicate_files: Vec<Vec<ResultFileInfo>>,
}

// Логика: у каждого плагина есть способ как представлять файл в виде хеша и как сравнивать эти представления чтобы получить результат формата PluginResult 
trait IDeduplicatorPlugin {
    fn name(&self) -> &str;
    fn exec<P: AsRef<Path>>(
        &self,
        file_list: &[P],
    ) -> Result<PluginResult, Box<dyn std::error::Error>>;
}

trait IPluginPipeline {
    fn add_plugin<T: IDeduplicatorPlugin>(&mut self, plugin: &T) -> Self;

    fn remove_plugin(&mut self, plugin_name: String) -> Self;

    fn list_plugins(&self);

    fn clear_list_plugins(&mut self);

    fn execute(&self, file_list: &Vec<&str>) -> Result<PluginResult, Box<dyn std::error::Error>>;
}


pub fn compute_md5(data: &[u8]) -> String {
    use md5::compute;
    format!("{:x}", compute(data))
}

// pub fn compute_sha256(data: &[u8]) -> String {
//     let mut context = Context::new(&SHA256);
//     context.update(data);
//     let digest = context.finish();
//     data_to_hex(digest.as_ref())
// }

pub fn compute_simple_hash(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

// Helper function to convert bytes to hex string
fn data_to_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}