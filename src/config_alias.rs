use std::collections::HashMap;

use anyhow::Result;

pub struct AliasConfig {
    pub map: crate::config_map::ConfigMap,
    pub parent: dyn crate::config::Config,
}

impl AliasConfig {
    pub fn get(&self, alias: &str) -> (String, bool) {
        if self.map.is_empty() {
            return ("".to_string(), false);
        }

        let value = match self.map.get_string_value(alias) {
            Ok(value) => value,
            Err(_) => "".to_string(),
        };

        (value.to_string(), !value.is_empty())
    }

    pub fn add(&mut self, alias: &str, expansion: &str) -> Result<()> {
        self.map.set_string_value(alias, expansion)?;

        // Update the parent config.
        self.parent.write()
    }

    pub fn delete(&mut self, alias: &str) -> Result<()> {
        self.map.remove_entry(alias)?;

        // Update the parent config.
        self.parent.write()
    }

    pub fn list(&self) -> HashMap<String, String> {
        let mut list: HashMap<String, String> = HashMap::new();

        for (key, value) in self.map.root.iter() {
            list.insert(key.to_string(), value.to_string());
        }

        list
    }
}
