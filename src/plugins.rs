use std::path::Path;

mod full_duplicates;
mod partial_duplicates;

#[derive(Debug)]
struct DuplicateGroup {
    group_number: usize,
    hash_list: Vec<String>,
}

#[derive(Debug)]
enum IsUnique {
    Unique,
    Duplicate(DuplicateGroup),
}

#[derive(Debug)]
struct ResultFileInfo {
    path: String,
    name: String,
    size: u64,
    hash: String,
    is_unique: IsUnique,
    plugin_type: String,
}

#[derive(Debug)]
struct PluginResult {
    unique_files: Vec<Result<ResultFileInfo, String>>,
    duplicate_files: Vec<Vec<Result<ResultFileInfo, String>>>,
}

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
