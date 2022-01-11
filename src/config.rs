use anyhow::{anyhow, Result};
use thiserror::Error;

/// This trait describes interaction with the configuration for oxide.
pub trait Config {
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

    /// Get the aliases.
    fn aliases(&self) -> Result<Vec<String>>;

    /// Check if the configuration can be written to.
    fn check_writable(&self) -> Result<()>;

    /// Write the configuration.
    fn write(&self) -> Result<()>;

    /// Return the string representation of the config.
    fn config_to_string(&self) -> Result<String>;

    /// Return the string representation of the hosts.
    fn hosts_to_string(&self) -> Result<String>;
}

pub struct ConfigOption {
    pub key: String,
    pub description: String,
    pub comment: String,
    pub default_value: String,
    pub allowed_values: Vec<String>,
}

pub fn config_options() -> Vec<ConfigOption> {
    vec![
        ConfigOption {
            key: "editor".to_string(),
            description: "the text editor program to use for authoring text".to_string(),
            comment: "What editor oxide should run when creating text, etc. If blank, will refer to environment."
                .to_string(),
            default_value: "".to_string(),
            allowed_values: vec![],
        },
        ConfigOption {
            key: "prompt".to_string(),
            description: "toggle interactive prompting in the terminal".to_string(),
            comment: "When to interactively prompt. This is a global config that cannot be overridden by hostname."
                .to_string(),
            default_value: "enabled".to_string(),
            allowed_values: vec!["enabled".to_string(), "disabled".to_string()],
        },
        ConfigOption {
            key: "pager".to_string(),
            description: "the terminal pager program to send standard output to".to_string(),
            comment: "A pager program to send command output to, e.g. \"less\". Set the value to \"cat\" to disable the pager.".to_string(),
            default_value: "".to_string(),
            allowed_values: vec![],
        },
        ConfigOption {
            key: "browser".to_string(),
            description: "the web browser to use for opening URLs".to_string(),
            comment: "What web browser gh should use when opening URLs. If blank, will refer to environment.".to_string(),
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

pub fn new_config(t: toml_edit::Document) -> impl Config {
    crate::config_from_file::FileConfig {
        map: crate::config_map::ConfigMap {
            root: t.as_table().clone(),
        },
    }
}

// new_from_string initializes a Config from a toml string.
fn new_from_string(s: &str) -> Result<impl Config> {
    let root = s.parse::<toml_edit::Document>()?;
    Ok(new_config(root))
}

pub fn new_blank_root() -> Result<toml_edit::Document> {
    let mut s = String::new();
    for option in config_options() {
        if !option.comment.is_empty() {
            s.push_str(&format!("# {}\n", option.comment));
            if !option.allowed_values.is_empty() {
                s.push_str(&format!("# Supported values: {}\n", option.allowed_values.join(", ")));
            }
        }
        s.push_str(&format!("{} = \"{}\"\n\n", option.key, option.default_value));
    }

    Ok(s.parse::<toml_edit::Document>()?)
}

pub fn new_blank_config() -> Result<impl Config> {
    let root = new_blank_root()?;
    Ok(new_config(root))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_file_config_set_no_host() {
        let mut c = new_blank_config().unwrap();
        assert!(c.set("", "editor", "vim").is_ok());
        assert!(c.set("", "prompt", "disabled").is_ok());
        assert!(c.set("", "pager", "less").is_ok());
        assert!(c.set("", "browser", "firefox").is_ok());

        let doc = c.config_to_string().unwrap();
        assert!(doc.contains("editor = \"vim\""));
        assert!(doc.contains("prompt = \"disabled\""));
        assert!(doc.contains("pager = \"less\""));
        assert!(doc.contains("browser = \"firefox\""));
    }

    #[test]
    fn test_file_config_set_with_host() {
        let mut c = new_blank_config().unwrap();
        assert!(c.set("example.com", "editor", "vim").is_ok());
        assert!(c.set("example.com", "prompt", "disabled").is_ok());
        assert!(c.set("example.com", "pager", "less").is_ok());
        assert!(c.set("example.com", "browser", "firefox").is_ok());
        assert!(c.set("oxide.computer", "browser", "chrome").is_ok());

        let doc = c.hosts_to_string().unwrap();

        let expected = r#"["example.com"]
browser = "firefox"

["oxide.computer"]
browser = "chrome""#;
        assert_eq!(doc, expected);
    }

    #[test]
    fn test_default_config() {
        let c = new_blank_config().unwrap();
        let doc_config = c.config_to_string().unwrap();

        let expected = r#"# What editor oxide should run when creating text, etc. If blank, will refer to environment.
editor = ""

# When to interactively prompt. This is a global config that cannot be overridden by hostname.
# Supported values: enabled, disabled
prompt = "enabled"

# A pager program to send command output to, e.g. "less". Set the value to "cat" to disable the pager.
pager = ""

# What web browser gh should use when opening URLs. If blank, will refer to environment.
browser = """#;
        assert_eq!(doc_config, expected);

        let doc_hosts = c.hosts_to_string().unwrap();
        assert_eq!(doc_hosts, "");
    }
}
