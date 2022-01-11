const OXIDE_HOST: &str = "OXIDE_HOST";
const OXIDE_TOKEN: &str = "OXIDE_TOKEN";

pub struct EnvConfig<'a> {
    pub config: &'a (dyn crate::config::Config + 'a),
}

/*impl crate::config::Config for EnvConfig {
    /// Returns a value from the configuration by its key.
    fn get(&self, hostname: &str, key: &str) -> Result<String>;
    /// Returns a value from the configuration by its key, with the source.
    fn get_with_source(&self, hostname: &str, key: &str) -> Result<(String, String)>;
    /// Sets a value in the configuration by its key.
    fn set(&mut self, hostname: &str, key: &str, value: &str) -> Result<()>;

    /// Remove a host.
    fn unset_host(&mut self, key: &str) -> Result<()>;
    /// Get the hosts.
    fn hosts(&self) -> Result<Vec<String>>;

    /// Get the default host.
    fn default_host(&self) -> Result<String>;
    // Get the default host with the source.
    fn default_host_with_source(&self) -> Result<(String, String)>;

    /// Get the aliases.
    fn aliases(&self) -> Result<crate::config_alias::AliasConfig>;

    /// Check if the configuration can be written to.
    fn check_writable(&self) -> Result<()>;

    /// Write the configuration.
    fn write(&self) -> Result<()>;

    /// Return the string representation of the config.
    fn config_to_string(&self) -> Result<String>;

    /// Return the string representation of the hosts.
    fn hosts_to_string(&self) -> Result<String>;
}*/
