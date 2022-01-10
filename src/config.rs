use anyhow::{anyhow, Result};
use thiserror::Error;

/// This trait describes interaction with the configuration for oxide.
pub trait Config {
    /// Returns a value from the configuration by its key.
    fn get(&self, key: &str) -> Result<String>;
    /// Returns a value from the configuration by its key, with the source.
    fn get_with_source(&self, key: &str) -> Result<(String, String)>;
    /// Sets a value in the configuration by its key.
    fn set(&mut self, key: &str, value: &str) -> Result<()>;

    fn unset_host(&mut self, key: &str) -> Result<()>;
    fn hosts(&self) -> Result<Vec<String>>;
    fn default_host(&self) -> Result<String>;

    fn aliases(&self) -> Result<Vec<String>>;

    fn check_writable(&self) -> Result<()>;

    fn write(&self) -> Result<()>;
}

pub struct ConfigOption {
    pub key: String,
    pub description: String,
    pub default_value: String,
    pub allowed_values: Vec<String>,
}

pub fn config_options() -> Vec<ConfigOption> {
    vec![
        ConfigOption {
            key: "editor".to_string(),
            description: "the text editor program to use for authoring text".to_string(),
            default_value: "".to_string(),
            allowed_values: vec![],
        },
        ConfigOption {
            key: "prompt".to_string(),
            description: "toggle interactive prompting in the terminal".to_string(),
            default_value: "enabled".to_string(),
            allowed_values: vec!["enabled".to_string(), "disabled".to_string()],
        },
        ConfigOption {
            key: "pager".to_string(),
            description: "the terminal pager program to send standard output to".to_string(),
            default_value: "".to_string(),
            allowed_values: vec![],
        },
        ConfigOption {
            key: "browser".to_string(),
            description: "the web browser to use for opening URLs".to_string(),
            default_value: "".to_string(),
            allowed_values: vec![],
        },
    ]
}

pub fn validate_key(key: String) -> Result<()> {
    for config_key in config_options() {
        if key == config_key.key {
            return Ok(());
        }
    }

    Err(anyhow!("invalid key"))
}

#[derive(Error, Debug)]
pub enum InvalidValueError {
    #[error("invalid values, valid values: {0:?}")]
    ValidValues(Vec<String>),
}

pub fn validate_value(key: String, value: String) -> Result<()> {
    let mut valid_values: Vec<String> = vec![];

    // Set the valid values for the key.
    for config_key in config_options() {
        if config_key.key == key {
            valid_values = config_key.allowed_values;
            break;
        }
    }

    if valid_values.is_empty() {
        return Ok(());
    }

    for v in valid_values.clone() {
        if v == value {
            return Ok(());
        }
    }

    Err(InvalidValueError::ValidValues(valid_values).into())
}

/*fn new_config(t: toml::Value) -> Config {}

// new_from_string initializes a Config from a toml string.
fn new_from_string(str: &str) -> Result<Config> {
    let root = toml::from_str(str)?;
    Ok(new_config(root))
}

pub fn new_blank_root() -> toml::Value {
    let root: Config = toml::from_str(
        r#"
        ip = '127.0.0.1'

        [keys]
        github = 'xxxxxxxxxxxxxxxxx'
        travis = 'yyyyyyyyyyyyyyyyy'
    "#,
    )
    .unwrap();
}*/
