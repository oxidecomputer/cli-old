use clap::Parser;

/// Create command shortcuts.
///
/// Aliases can be used to make shortcuts for oxide commands or to compose multiple commands.
/// Run "oxide help alias set" to learn more.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAlias {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Delete(CmdAliasDelete),
    Set(CmdAliasSet),
    List(CmdAliasList),
}

impl CmdAlias {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        match &self.subcmd {
            SubCommand::Delete(cmd) => cmd.run(config),
            SubCommand::Set(cmd) => cmd.run(config),
            SubCommand::List(cmd) => cmd.run(config),
        }
    }
}

/// Print the value of a given configuration key.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAliasDelete {
    #[clap(name = "alias", required = true)]
    alias: String,
}

impl CmdAliasDelete {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        let mut alias_config = config.aliases().unwrap();

        let (expansion, ok) = alias_config.get(&self.alias);
        if !ok {
            eprintln!("no such alias {}", self.alias);
            std::process::exit(1);
        }

        match alias_config.delete(&self.alias) {
            Ok(_) => {
                println!("Deleted alias {}; was {}", self.alias, expansion);
            }
            Err(e) => {
                eprintln!("failed to delete alias {}: {}", self.alias, e);
                std::process::exit(1);
            }
        }
    }
}

/// Update configuration with a value for the given key.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAliasSet {
    #[clap(name = "key", required = true)]
    key: String,

    #[clap(name = "value", required = true)]
    value: String,

    /// Set per-host setting.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
}

impl CmdAliasSet {
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
pub struct CmdAliasList {
    /// Get per-host configuration.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
}

impl CmdAliasList {
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
