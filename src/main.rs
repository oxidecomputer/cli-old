//! The Oxide command line tool.
#![deny(missing_docs)]

mod cmd_alias;
mod cmd_completion;
mod cmd_config;
mod colors;
mod config;
mod config_alias;
mod config_file;
mod config_from_env;
mod config_from_file;
mod config_map;
mod iostreams;

use clap::Parser;

use crate::config::Config;

/// Work seamlessly with Oxide from the command line.
///
/// Environment variables that can be used with oxide
///
/// OXIDE_TOKEN, OXIDE_API_TOKEN (in order of precedence): an authentication token
/// for Oxide API requests. Setting this avoids being prompted to authenticate
/// and takes precedence over previously stored credentials.
///
/// OXIDE_HOST: specify the Oxide hostname for commands that would otherwise assume
/// the "api.oxide.computer" host.
///
/// OXIDE_BROWSER, BROWSER (in order of precedence): the web browser to use for opening
/// links.
///
/// DEBUG: set to any value to enable verbose output to standard error.
///
/// OXIDE_PAGER, PAGER (in order of precedence): a terminal paging program to send
/// standard output to, e.g. "less".
///
/// GLAMOUR_STYLE: the style to use for rendering Markdown. See
/// <https://github.com/charmbracelet/glamour#styles>
///
/// NO_COLOR: set to any value to avoid printing ANSI escape sequences for color output.
///
/// CLICOLOR: set to "0" to disable printing ANSI colors in output.
///
/// CLICOLOR_FORCE: set to a value other than "0" to keep ANSI colors in output
/// even when the output is piped.
///
/// OXIDE_FORCE_TTY: set to any value to force terminal-style output even when the
/// output is redirected. When the value is a number, it is interpreted as the number of
/// columns available in the viewport. When the value is a percentage, it will be applied
/// against the number of columns available in the current viewport.
///
/// OXIDE_NO_UPDATE_NOTIFIER: set to any value to disable update notifications. By
/// default, oxide checks for new releases once every 24 hours and displays an upgrade
/// notice on standard error if a newer version was found.
///
/// OXIDE_CONFIG_DIR: the directory where oxide will store configuration files.
/// Default: "$XDG_CONFIG_HOME/oxide" or "$HOME/.config/oxide".
#[derive(Parser, Debug, Clone)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!("\n"))]
struct Opts {
    /// Print debug info
    #[clap(short, long)]
    debug: bool,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Alias(cmd_alias::CmdAlias),
    Completion(cmd_completion::CmdCompletion),
    Config(cmd_config::CmdConfig),
}

fn main() {
    // Parse the command line arguments.
    let opts: Opts = Opts::parse();

    // Let's get our configuration.
    let mut c = crate::config_file::parse_default_config().unwrap();
    let mut config = crate::config_from_env::EnvConfig::inherit_env(&mut c);

    // Let's get our IO streams.
    let mut io = crate::iostreams::IoStreams::system();

    let prompt = config.get("", "prompt").unwrap();
    if prompt == "disabled" {
        io.set_never_prompt(true)
    }

    // Pager precedence
    // 1. OXIDE_PAGER
    // 2. pager from config
    // 3. PAGER
    if let Ok(oxide_pager) = std::env::var("OXIDE_PAGER") {
        io.set_pager(oxide_pager);
    } else if let Ok(pager) = config.get("", "pager") {
        if !pager.is_empty() {
            io.set_pager(pager);
        }
    }

    match opts.subcmd {
        SubCommand::Alias(cmd) => cmd.run(&mut config),
        SubCommand::Completion(cmd) => {
            cmd.run();
        }
        SubCommand::Config(cmd) => {
            cmd.run(&mut config);
        }
    }
}
