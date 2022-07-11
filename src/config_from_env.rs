use std::env;

use anyhow::Result;
use thiserror::Error;

use crate::cmd_auth::parse_host;
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

#[derive(Error, Debug)]
pub enum ReadOnlyEnvVarError {
    #[error("read-only value in: {0}")]
    Variable(String),
}

unsafe impl Send for EnvConfig<'_> {}
unsafe impl Sync for EnvConfig<'_> {}

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
        } else {
            let var = format!("OXIDE_{}", heck::AsShoutySnakeCase(key));
            let val = get_env_var(&var);
            if !val.is_empty() {
                return Ok((val, var));
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
            let host = parse_host(&host)?;
            Ok((host.to_string(), OXIDE_HOST.to_string()))
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

    fn expand_alias(&mut self, args: Vec<String>) -> Result<(Vec<String>, bool)> {
        self.config.expand_alias(args)
    }

    fn check_writable(&self, hostname: &str, key: &str) -> Result<()> {
        // If they are asking specifically for the token, return the value.
        if key == "token" {
            let token = get_env_var(OXIDE_TOKEN);
            if !token.is_empty() {
                return Err(ReadOnlyEnvVarError::Variable(OXIDE_TOKEN.to_string()).into());
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
