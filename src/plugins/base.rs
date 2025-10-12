trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self, input: &str) -> String;
    fn version(&self) -> String {
        String::from("1.0.0")
    }
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    fn execute_all(&self, input: &str) -> Vec<String> {
        self.plugins.iter().map(|p| p.execute(input)).collect()
    }

    fn list_plugins(&self) {
        for plugin in &self.plugins {
            println!("{} v{}", plugin.name(), plugin.version());
        }
    }
}
