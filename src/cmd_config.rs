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

impl crate::cmd::Command for CmdConfig {
    fn run(&self, ctx: crate::context::Context) {
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

impl crate::cmd::Command for CmdConfigGet {
    fn run(&self, mut ctx: crate::context::Context) {
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

impl crate::cmd::Command for CmdConfigSet {
    fn run(&self, mut ctx: crate::context::Context) {
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

impl crate::cmd::Command for CmdConfigList {
    fn run(&self, mut ctx: crate::context::Context) {
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
    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        input: String,
        want_out: String,
        want_err: String,
    }

    #[test]
    fn test_cmd_config_get() {
        let tests: Vec<TestItem> = vec![
            /*TestItem {
                name: "get key".to_string(),
                input: "key".to_string(),
                want_out: "".to_string(),
                want_err: "Key 'key' not found".to_string(),
            },
            TestItem {
                name: "get key with host".to_string(),
                input: "test".to_string(),
                want_out: "".to_string(),
                want_err: "".to_string(),
            },*/
        ];

        for t in tests {
            let cmd = crate::cmd_config::CmdConfigGet {
                host: "".to_string(),
                key: t.input.to_string(),
            };

            let (io, stdout_path, _) = crate::iostreams::IoStreams::test();
            let mut config = crate::config::new_blank_config().unwrap();
            let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);
            let ctx = crate::context::Context { config: &mut c, io };

            cmd.run(ctx);
            let s = std::fs::read_to_string(&stdout_path).unwrap();

            assert!(s.contains(&t.want_out), "test {}", t.name);
        }
    }
}
