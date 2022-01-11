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
    pub fn run(&self, ctx: crate::context::Context) {
        match &self.subcmd {
            SubCommand::Get(cmd) => cmd.run(ctx),
            SubCommand::Set(cmd) => cmd.run(ctx),
            SubCommand::List(cmd) => cmd.run(ctx),
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
    pub fn run(&self, mut ctx: crate::context::Context) {
        match ctx.config.get(&self.host, &self.key) {
            Ok(value) => writeln!(ctx.io.out, "{}", value).unwrap(),
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
    pub fn run(&self, mut ctx: crate::context::Context) {
        let cs = ctx.io.color_scheme();

        // Validate the key.
        match crate::config::validate_key(&self.key) {
            Ok(()) => (),
            Err(_) => {
                writeln!(
                    ctx.io.err_out,
                    "{} warning: '{}' is not a known configuration key",
                    cs.warning_icon(),
                    self.key
                )
                .unwrap();
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
        match ctx.config.set(&self.host, &self.key, &self.value) {
            Ok(()) => (),
            Err(err) => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        }

        // Write the config file.
        match ctx.config.write() {
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
    pub fn run(&self, mut ctx: crate::context::Context) {
        let host = if self.host.is_empty() {
            ctx.config.default_host().unwrap_or_default()
        } else {
            self.host.to_string()
        };

        for option in crate::config::config_options() {
            match ctx.config.get(&host, &option.key) {
                Ok(value) => writeln!(ctx.io.out, "{}={}", option.key, value).unwrap(),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    pub struct TestItem {
        name: String,
        input: String,
        out: String,
        wants_err: bool,
    }

    #[test]
    fn test_cmd_config_get() {
        let tests = vec![
            TestItem {
                name: "no arguments".to_string(),
                input: "".to_string(),
                out: "".to_string(),
                wants_err: true,
            },
            TestItem {
                name: "get key".to_string(),
                input: "key".to_string(),
                out: "thing".to_string(),
                wants_err: false,
            },
            TestItem {
                name: "get key with host".to_string(),
                input: "key --host test.com".to_string(),
                out: "".to_string(),
                wants_err: false,
            },
        ];

        for _t in tests {}
    }
}
