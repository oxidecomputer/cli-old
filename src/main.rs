//! The Oxide command line tool.
#![deny(missing_docs)]

// Always export the cmd_* modules as public so that it tells us when we are
// missing docs.

mod cmd;
/// The alias command.
pub mod cmd_alias;
/// The completion command.
pub mod cmd_completion;
/// The config command.
pub mod cmd_config;
/// The generate command.
pub mod cmd_generate;
mod colors;
mod config;
mod config_alias;
mod config_file;
mod config_from_env;
mod config_from_file;
mod config_map;
mod context;
mod iostreams;
mod man;
mod markdown;

use clap::Parser;
use config::Config;

/// Work seamlessly with Oxide from the command line.
///
/// Environment variables that can be used with oxide.
///
/// OXIDE_TOKEN: an authentication token for Oxide API requests. Setting this
/// avoids being prompted to authenticate and takes precedence over previously
/// stored credentials.
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
    #[clap(short, long, global = true, env)]
    debug: bool,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Alias(cmd_alias::CmdAlias),
    Completion(cmd_completion::CmdCompletion),
    Config(cmd_config::CmdConfig),
    Generate(cmd_generate::CmdGenerate),
}

fn main() {
    // Let's get our configuration.
    let mut c = crate::config_file::parse_default_config().unwrap();
    let mut config = crate::config_from_env::EnvConfig::inherit_env(&mut c);
    let mut ctx = crate::context::Context::new(&mut config);

    // Let's grab all our args.
    let mut args: Vec<String> = std::env::args().collect();
    let args_str = shlex::join(args.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

    /*let first_arg = if args.len() > 1 {
        args.get(1).unwrap().to_string()
    } else {
        "".to_string()
    };

    // Convert our opts into a clap app.
    let app: clap::App = crate::Opts::into_app();*/

    // Check if the user is passing in an alias.
    if !crate::cmd_alias::valid_command(&args_str) {
        // Let's validate if it is an alias.
        // It is okay to check the error here because we will not error out if the
        // alias does not exist. We will just return the expanded args.
        let (mut expanded_args, is_shell) = ctx.config.expand_alias(args).unwrap();

        // TODO: debug
        //if opts.debug {
        writeln!(
            &mut ctx.io.out,
            "Expanded alias: {} -> {}",
            args_str,
            &shlex::join(expanded_args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
        )
        .unwrap();
        //}

        if is_shell {
            // Remove the first argument, since thats our `sh`.
            expanded_args.remove(0);

            let mut external_cmd = std::process::Command::new("sh").args(expanded_args).spawn().unwrap();
            let ecode = external_cmd.wait().unwrap();
            std::process::exit(ecode.code().unwrap_or(0));
        }

        // So we handled if the alias was a shell.
        // We can now parse our options from the extended args.
        args = expanded_args;
    }

    // Parse the command line arguments.
    let opts: Opts = Opts::parse_from(args);

    // Set our debug flag.
    ctx.debug = opts.debug;

    match opts.subcmd {
        SubCommand::Alias(cmd) => run_cmd(&cmd, &mut ctx),
        SubCommand::Completion(cmd) => run_cmd(&cmd, &mut ctx),
        SubCommand::Config(cmd) => run_cmd(&cmd, &mut ctx),
        SubCommand::Generate(cmd) => run_cmd(&cmd, &mut ctx),
    }
}

fn run_cmd(cmd: &impl crate::cmd::Command, ctx: &mut context::Context) {
    if let Err(err) = cmd.run(ctx) {
        writeln!(ctx.io.err_out, "{}", err).unwrap();
        std::process::exit(1);
    }
}
