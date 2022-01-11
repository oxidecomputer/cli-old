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
    pub fn run(&self) {
        match &self.subcmd {
            SubCommand::Get(cmd) => cmd.run(),
            SubCommand::Set(cmd) => cmd.run(),
            SubCommand::List(cmd) => cmd.run(),
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
    pub fn run(&self) {}
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
    pub fn run(&self) {}
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
    pub fn run(&self) {}
}
