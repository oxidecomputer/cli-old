use clap::Parser;

/// Manage configuration for oxide.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdConfig {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Get(CmdConfigGet),
    Set(CmdConfigSet),
    List(CmdConfigList),
}

impl CmdConfig {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        match &self.subcmd {
            SubCommand::Get(cmd) => cmd.run(config),
            SubCommand::Set(cmd) => cmd.run(config),
            SubCommand::List(cmd) => cmd.run(config),
        }
    }
}

/// Print the value of a given configuration key.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdConfigGet {
    #[clap(name = "key", required = true)]
    key: String,

    /// Get per-host setting.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
}

impl CmdConfigGet {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        match config.get(&self.host, &self.key) {
            Ok(value) => println!("{}", value),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    }
}

/// Update configuration with a value for the given key.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdConfigSet {
    #[clap(name = "key", required = true)]
    key: String,

    #[clap(name = "value", required = true)]
    value: String,

    /// Set per-host setting.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
}

impl CmdConfigSet {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        // Validate the key.
        match crate::config::validate_key(&self.key) {
            Ok(()) => (),
            Err(_) => {
                eprintln!("warning: '{}' is not a known configuration key", self.key);
                std::process::exit(1);
            }
        }

        // Validate the value.
        match crate::config::validate_value(&self.key, &self.value) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }

        // Set the value.
        match config.set(&self.host, &self.key, &self.value) {
            Ok(()) => (),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }

        // Write the config file.
        match config.write() {
            Ok(()) => (),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }
    }
}

/// Print a list of configuration keys and values.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdConfigList {
    /// Get per-host configuration.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
}

impl CmdConfigList {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        let host = if self.host.is_empty() {
            config.default_host().unwrap_or_default()
        } else {
            self.host.to_string()
        };

        for option in crate::config::config_options() {
            match config.get(&host, &option.key) {
                Ok(value) => println!("{}={}", option.key, value),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}
