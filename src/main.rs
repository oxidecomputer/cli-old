//! The Oxide command line tool.
#![deny(missing_docs)]

// Always export the cmd_* modules as public so that it tells us when we are
// missing docs.

mod cmd;
/// The alias command.
pub mod cmd_alias;
/// The api command.
pub mod cmd_api;
/// The auth command.
pub mod cmd_auth;
/// The completion command.
pub mod cmd_completion;
/// The config command.
pub mod cmd_config;
/// The disk command.
pub mod cmd_disk;
/// The generate command.
pub mod cmd_generate;
/// The image command.
pub mod cmd_image;
/// The image global subcommand.
pub mod cmd_image_global;
/// The instance command.
pub mod cmd_instance;
#[cfg(unix)]
/// Support for interactive instance serial access
pub mod cmd_instance_serial;
/// The open command.
pub mod cmd_open;
/// The organization command.
pub mod cmd_org;
/// The project command.
pub mod cmd_project;
/// The rack command.
pub mod cmd_rack;
/// The role command.
pub mod cmd_role;
/// The route command.
pub mod cmd_route;
/// The router command.
pub mod cmd_router;
/// The sled command.
pub mod cmd_sled;
/// The snapshot command.
pub mod cmd_snapshot;
/// The ssh-key command.
pub mod cmd_ssh_key;
/// The subnet command.
pub mod cmd_subnet;
/// The update command.
pub mod cmd_update;
/// The version command.
pub mod cmd_version;
/// The vpc command.
pub mod cmd_vpc;

// Use of a mod or pub mod is not actually necessary.
mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

mod colors;
mod config;
mod config_alias;
mod config_file;
mod config_from_env;
mod config_from_file;
mod config_map;
mod context;
mod docs_man;
mod docs_markdown;
mod iostreams;
mod prompt_ext;
mod types;

#[cfg(test)]
mod tests;

mod update;

use std::io::{Read, Write};

use anyhow::Result;
use clap::Parser;
use slog::Drain;

/// Work seamlessly with Oxide from the command line.
///
/// Environment variables that can be used with oxide. Additionally to those
/// listed below, some flags have a corresponding environment variable. For example,
/// most of the time, the `--organization,-o` flag is mapped to the `OXIDE_ORG` environment
/// variable.
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
///
/// Authentication
///
/// You can get an access token running `oxide auth login`. This will contact `OXIDE_HOST`
/// and attempt an OAuth 2.0 Device Authorization Grant.
/// The CLI will attempt to open a browser window with which you can login
/// (via SAML or other IdP method) and type in or verify the user code printed in the terminal.
/// After a successful login and code verification, a token associated with the logged-in
/// user will be granted and stored in the config file.
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
    #[clap(alias = "aliases")]
    Alias(cmd_alias::CmdAlias),
    Api(cmd_api::CmdApi),
    Auth(cmd_auth::CmdAuth),
    Completion(cmd_completion::CmdCompletion),
    Config(cmd_config::CmdConfig),
    #[clap(alias = "disks")]
    Disk(cmd_disk::CmdDisk),
    Generate(cmd_generate::CmdGenerate),
    #[clap(alias = "images")]
    Image(cmd_image::CmdImage),
    #[clap(alias = "instances")]
    Instance(cmd_instance::CmdInstance),
    #[clap(alias = "open")]
    Open(cmd_open::CmdOpen),
    #[clap(alias = "orgs")]
    Org(cmd_org::CmdOrganization),
    #[clap(alias = "projects")]
    Project(cmd_project::CmdProject),
    #[clap(alias = "racks")]
    Rack(cmd_rack::CmdRack),
    #[clap(alias = "roles")]
    Role(cmd_role::CmdRole),
    #[clap(alias = "routes")]
    Route(cmd_route::CmdRoute),
    #[clap(alias = "routers")]
    Router(cmd_router::CmdRouter),
    #[clap(alias = "sleds")]
    Sled(cmd_sled::CmdSled),
    #[clap(alias = "snapshots")]
    Snapshot(cmd_snapshot::CmdSnapshot),
    #[clap(alias = "ssh-keys")]
    SshKey(cmd_ssh_key::CmdSSHKey),
    #[clap(alias = "subnets")]
    Subnet(cmd_subnet::CmdSubnet),
    Update(cmd_update::CmdUpdate),
    Version(cmd_version::CmdVersion),
    #[clap(alias = "vpcs")]
    Vpc(cmd_vpc::CmdVpc),
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let build_version = clap::crate_version!();
    // Check for updates to the cli.
    // We don't await here since we don't want to block the main thread.
    // We'll check again before we exit.
    let update = crate::update::check_for_update(build_version, false);

    // Let's get our configuration.
    let mut c = crate::config_file::parse_default_config().unwrap();
    let mut config = crate::config_from_env::EnvConfig::inherit_env(&mut c);
    let mut ctx = crate::context::Context::new(&mut config);

    // Let's grab all our args.
    let args: Vec<String> = std::env::args().collect();
    let result = do_main(args, &mut ctx).await;

    // If we have an update, let's print it.
    handle_update(&mut ctx, update.await.unwrap_or_default(), build_version).unwrap();

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }

    std::process::exit(result.unwrap_or(0));
}

async fn do_main(mut args: Vec<String>, ctx: &mut crate::context::Context<'_>) -> Result<i32> {
    let original_args = args.clone();

    // Remove the first argument, which is the program name, and can change depending on how
    // they are calling it.
    args.remove(0);

    let args_str = shlex::join(args.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

    // Check if the user is passing in an alias.
    if !crate::cmd_alias::valid_command(&args_str) {
        // Let's validate if it is an alias.
        // It is okay to check the error here because we will not error out if the
        // alias does not exist. We will just return the expanded args.
        let (mut expanded_args, is_shell) = ctx.config.expand_alias(original_args)?;

        if is_shell {
            // Remove the first argument, since thats our `sh`.
            expanded_args.remove(0);

            let mut external_cmd = std::process::Command::new("sh")
                .args(expanded_args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()?;

            let ecode = external_cmd.wait()?;

            // Pipe the output to the terminal.
            if let Some(stdout_rd) = external_cmd.stdout.as_mut() {
                let mut stdout = Vec::new();
                stdout_rd.read_to_end(&mut stdout)?;
                ctx.io.out.write_all(&stdout)?;
            }

            if let Some(mut stderr_rd) = external_cmd.stderr {
                let mut stderr = Vec::new();
                stderr_rd.read_to_end(&mut stderr)?;
                ctx.io.err_out.write_all(&stderr)?;
            }

            return Ok(ecode.code().unwrap_or(0));
        }

        // So we handled if the alias was a shell.
        // We can now parse our options from the extended args.
        args = expanded_args;
    } else {
        args = original_args;
    }

    // Parse the command line arguments.
    let opts: Opts = Opts::parse_from(args);

    // Set our debug flag.
    ctx.debug = opts.debug;

    // Setup our logger. This is mainly for debug purposes.
    // And getting debug logs from other libraries we consume, like even Oxide.
    if ctx.debug {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        let logger = slog::Logger::root(drain, slog::o!());

        let scope_guard = slog_scope::set_global_logger(logger);
        scope_guard.cancel_reset();

        slog_stdlog::init_with_level(log::Level::Debug).unwrap();
    }

    match opts.subcmd {
        SubCommand::Alias(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Api(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Auth(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Completion(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Config(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Disk(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Generate(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Image(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Instance(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Open(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Org(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Project(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Rack(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Role(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Route(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Router(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Sled(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Snapshot(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::SshKey(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Subnet(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Update(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Version(cmd) => run_cmd(&cmd, ctx).await,
        SubCommand::Vpc(cmd) => run_cmd(&cmd, ctx).await,
    }
}

async fn run_cmd(cmd: &impl crate::cmd::Command, ctx: &mut context::Context<'_>) -> Result<i32> {
    let cs = ctx.io.color_scheme();

    if let Err(err) = cmd.run(ctx).await {
        // If the error was from the API, let's handle it better for each type of error.
        // These are defined here: https://github.com/oxidecomputer/omicron/blob/main/common/src/api/external/error.rs#L28
        match err.downcast_ref::<oxide_api::types::Error>() {
            Some(oxide_api::types::Error::ObjectNotFound { message }) => {
                writeln!(ctx.io.err_out, "{} Object not found: {}", cs.failure_icon(), message,)?;
            }
            Some(oxide_api::types::Error::ObjectAlreadyExists { message }) => {
                writeln!(
                    ctx.io.err_out,
                    "{} Object already exists: {}",
                    cs.failure_icon(),
                    message,
                )?;
            }
            Some(oxide_api::types::Error::InvalidRequest { message }) => {
                writeln!(ctx.io.err_out, "{} Invalid request: {}", cs.failure_icon(), message,)?;
            }
            Some(oxide_api::types::Error::Unauthenticated { internal_message }) => {
                writeln!(
                    ctx.io.err_out,
                    "{} You are not authenticated: {}",
                    cs.failure_icon(),
                    internal_message
                )?;

                writeln!(ctx.io.err_out, "Try authenticating with: `oxide auth login`")?;
            }
            Some(oxide_api::types::Error::InvalidValue { message }) => {
                writeln!(ctx.io.err_out, "{} Invalid value: {}", cs.failure_icon(), message)?;
            }
            Some(oxide_api::types::Error::Forbidden) => {
                writeln!(
                    ctx.io.err_out,
                    "{} You are not authorized to perform this action",
                    cs.failure_icon(),
                )?;
            }
            Some(oxide_api::types::Error::InternalError { internal_message }) => {
                writeln!(
                    ctx.io.err_out,
                    "{} Oxide API internal error: {}",
                    cs.failure_icon(),
                    internal_message
                )?;
            }
            Some(oxide_api::types::Error::ServiceUnavailable { internal_message }) => {
                writeln!(
                    ctx.io.err_out,
                    "{} Oxide API service unavailable: {}",
                    cs.failure_icon(),
                    internal_message
                )?;
            }
            Some(oxide_api::types::Error::MethodNotAllowed { internal_message }) => {
                writeln!(
                    ctx.io.err_out,
                    "{} Oxide API method not allowed: {}",
                    cs.failure_icon(),
                    internal_message
                )?;
            }
            None => {
                writeln!(ctx.io.err_out, "{}", err)?;
            }
        }
        return Ok(1);
    }

    Ok(0)
}

fn handle_update(
    ctx: &mut crate::context::Context,
    update: Option<crate::update::ReleaseInfo>,
    build_version: &str,
) -> Result<()> {
    if let Some(latest_release) = update {
        // do not notify Homebrew users before the version bump had a chance to get merged into homebrew-core
        let is_homebrew = crate::update::is_under_homebrew()?;

        if !(is_homebrew && crate::update::is_recent_release(latest_release.published_at)) {
            let cs = ctx.io.color_scheme();

            writeln!(
                ctx.io.err_out,
                "\n\n{} {} → {}\n",
                cs.yellow("A new release of oxide is available:"),
                cs.cyan(build_version),
                cs.purple(&latest_release.version)
            )?;

            if is_homebrew {
                writeln!(ctx.io.err_out, "To upgrade, run: `brew update && brew upgrade oxide`")?;
            } else {
                writeln!(ctx.io.err_out, "To upgrade, run: `oxide update`")?;
            }

            writeln!(ctx.io.err_out, "{}\n\n", cs.yellow(&latest_release.url))?;
        }
    }

    Ok(())
}
