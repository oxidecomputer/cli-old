use anyhow::{bail, Result};
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
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
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
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match ctx.config.get(&self.host, &self.key) {
            Ok(value) => writeln!(ctx.io.out, "{}", value).unwrap(),
            Err(err) => {
                bail!("{}", err);
            }
        }

        Ok(())
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
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let cs = ctx.io.color_scheme();

        // Validate the key.
        match crate::config::validate_key(&self.key) {
            Ok(()) => (),
            Err(_) => {
                bail!(
                    "{} warning: '{}' is not a known configuration key",
                    cs.warning_icon(),
                    self.key
                );
            }
        }

        // Validate the value.
        if let Err(err) = crate::config::validate_value(&self.key, &self.value) {
            bail!("{}", err);
        }

        // Set the value.
        if let Err(err) = ctx.config.set(&self.host, &self.key, &self.value) {
            bail!("{}", err);
        }

        // Write the config file.
        if let Err(err) = ctx.config.write() {
            bail!("{}", err);
        }

        Ok(())
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
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let host = if self.host.is_empty() {
            ctx.config.default_host().unwrap_or_default()
        } else {
            self.host.to_string()
        };

        for option in crate::config::config_options() {
            match ctx.config.get(&host, &option.key) {
                Ok(value) => writeln!(ctx.io.out, "{}={}", option.key, value).unwrap(),
                Err(err) => {
                    bail!("{}", err);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        cmd: crate::cmd_config::SubCommand,
        want_out: String,
        want_err: String,
    }

    #[test]
    fn test_cmd_config() {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "set a key".to_string(),
                cmd: crate::cmd_config::SubCommand::Set(crate::cmd_config::CmdConfigSet {
                    key: "foo".to_string(),
                    value: "bar".to_string(),
                    host: "".to_string(),
                }),
                want_out: "".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "set a key with host".to_string(),
                cmd: crate::cmd_config::SubCommand::Set(crate::cmd_config::CmdConfigSet {
                    key: "weird".to_string(),
                    value: "science".to_string(),
                    host: "example.org".to_string(),
                }),
                want_out: "".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "get a key we set".to_string(),
                cmd: crate::cmd_config::SubCommand::Get(crate::cmd_config::CmdConfigGet {
                    key: "foo".to_string(),
                    host: "".to_string(),
                }),
                want_out: "".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "get a key we set with host".to_string(),
                cmd: crate::cmd_config::SubCommand::Get(crate::cmd_config::CmdConfigGet {
                    key: "weird".to_string(),
                    host: "example.org".to_string(),
                }),
                want_out: "".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "get a non existent key".to_string(),
                cmd: crate::cmd_config::SubCommand::Get(crate::cmd_config::CmdConfigGet {
                    key: "blah".to_string(),
                    host: "".to_string(),
                }),
                want_out: "".to_string(),
                want_err: "".to_string(),
            },
        ];

        for t in tests {
            let (io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
            let mut config = crate::config::new_blank_config().unwrap();
            let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);
            let mut ctx = crate::context::Context { config: &mut c, io };

            let cmd_config = crate::cmd_config::CmdConfig { subcmd: t.cmd };
            match cmd_config.run(&mut ctx) {
                Ok(()) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert_eq!(stdout, t.want_out, "test {}", t.name);
                    assert_eq!(stderr, t.want_err, "test {}", t.name);
                }
                Err(err) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert_eq!(stdout, t.want_out, "test {}", t.name);
                    assert_eq!(stderr, t.want_err, "test {}", t.name);
                }
            }
        }
    }
}
