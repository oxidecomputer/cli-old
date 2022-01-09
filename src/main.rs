//! The Oxide command line tool.
#![deny(missing_docs)]

mod completion;

use clap::{App, IntoApp, Parser};
use clap_complete::generate;

use std::io;

/// Work seamlessly with Oxide from the command line.
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
    Completion(completion::CmdCompletion),
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
