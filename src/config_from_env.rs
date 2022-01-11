use std::env;

use anyhow::Result;

use crate::config_file::get_env_var;

const OXIDE_HOST: &str = "OXIDE_HOST";
const OXIDE_TOKEN: &str = "OXIDE_TOKEN";

pub struct EnvConfig<'a> {
    pub config: &'a mut (dyn crate::config::Config + 'a),
}

impl EnvConfig<'_> {
    pub fn inherit_env(config: &mut dyn crate::config::Config) -> EnvConfig {
        EnvConfig { config }
    }
}

impl crate::config::Config for EnvConfig<'_> {
    fn get(&self, hostname: &str, key: &str) -> Result<String> {
        let (val, _) = self.get_with_source(hostname, key)?;
        Ok(val)
    }

    fn get_with_source(&self, hostname: &str, key: &str) -> Result<(String, String)> {
        // If they are asking specifically for the token, return the value.
        if key == "token" {
            let token = get_env_var(OXIDE_TOKEN);
            if !token.is_empty() {
                return Ok((token, OXIDE_TOKEN.to_string()));
            }
        }

        self.config.get_with_source(hostname, key)
    }

    fn set(&mut self, hostname: &str, key: &str, value: &str) -> Result<()> {
        self.config.set(hostname, key, value)
    }

    fn unset_host(&mut self, key: &str) -> Result<()> {
        self.config.unset_host(key)
    }

    fn hosts(&self) -> Result<Vec<String>> {
        self.config.hosts()
    }

    fn default_host(&self) -> Result<String> {
        let (host, _) = self.default_host_with_source()?;
        Ok(host)
    }

    fn default_host_with_source(&self) -> Result<(String, String)> {
        if let Ok(host) = env::var(OXIDE_HOST) {
            Ok((host, OXIDE_HOST.to_string()))
        } else {
            self.config.default_host_with_source()
        }
    }

    fn aliases(&mut self) -> Result<crate::config_alias::AliasConfig> {
        self.config.aliases()
    }

    fn save_aliases(&mut self, aliases: &crate::config_map::ConfigMap) -> Result<()> {
        self.config.save_aliases(aliases)
    }

    fn check_writable(&self, hostname: &str, key: &str) -> Result<()> {
        // If they are asking specifically for the token, return the value.
        if key == "token" {
            let token = get_env_var(OXIDE_TOKEN);
            if !token.is_empty() {
                return Err(anyhow::anyhow!("Cannot write to env var {}", OXIDE_TOKEN));
            }
        }

        self.config.check_writable(hostname, key)
    }

    fn write(&self) -> Result<()> {
        self.config.write()
    }

    fn config_to_string(&self) -> Result<String> {
        self.config.config_to_string()
    }

    fn hosts_to_string(&self) -> Result<String> {
        self.config.hosts_to_string()
    }
}
