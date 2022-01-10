//! The Oxide command line tool.
#![deny(missing_docs)]

mod cmd_completion;
mod config;
mod config_file;

use clap::{App, IntoApp, Parser};
use clap_complete::generate;

use std::io;

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
    Completion(cmd_completion::CmdCompletion),
}

fn main() {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Completion(cmd) => {
            // Convert our opts into a clap app.
            let mut app: App = Opts::into_app();
            let name = app.get_name().to_string();
            // Generate the completion script.
            generate(cmd.shell, &mut app, name, &mut io::stdout());
        }
    }
}
