use anyhow::{anyhow, Result};

// This type implements a Config interface and represents a config file on disk.
pub struct FileConfig {
    pub map: crate::config_map::ConfigMap,
}

pub struct HostConfig {
    pub map: crate::config_map::ConfigMap,
    pub host: String,
}

impl crate::config::Config for FileConfig {
    /// Returns a value from the configuration by its key.
    fn get(&self, key: &str) -> Result<String> {
        let (val, _) = self.get_with_source(key)?;
        Ok(val)
    }

    /// Returns a value from the configuration by its key, with the source.
    fn get_with_source(&self, key: &str) -> Result<(String, String)> {
        let default_source = crate::config_file::config_file()?;
        let value = self.map.get_string_value(key)?;

        Ok((value, default_source))
    }

    /// Sets a value in the configuration by its key.
    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.map.set_string_value(key, value)
    }

    fn unset_host(&mut self, hostname: &str) -> Result<()> {
        if hostname.is_empty() {
            return Ok(());
        }

        let hosts = self.map.find_entry("hosts")?;
        // Parse the hosts as an array of tables.
        match hosts.as_table() {
            Some(h) => {
                let mut hosts_table = h.clone();
                // Remove the host from the table.
                hosts_table.remove_entry(hostname);
                Ok(())
            }
            None => {
                return Err(anyhow!("hosts is not an array of tables, cannot unset host"));
            }
        }
    }
    fn hosts(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
    fn default_host(&self) -> Result<String> {
        Ok("".to_string())
    }

    fn aliases(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    fn check_writable(&self) -> Result<()> {
        // TODO: check if the config file is writable from the filesystem permissions
        Ok(())
    }

    fn write(&self) -> Result<()> {
        // Get the config file name.
        let filename = crate::config_file::config_file()?;

        // Get the string representation of the config file.
        let content = self.map.root.to_string();

        // Write the config file.
        crate::config_file::write_config_file(&filename, &content)
    }
}
