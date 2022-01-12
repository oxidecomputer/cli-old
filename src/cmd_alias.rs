use std::io::Write;

use anyhow::{bail, Result};
use clap::{App, IntoApp, Parser};

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

impl crate::cmd::Command for CmdAlias {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Delete(cmd) => cmd.run(ctx),
            SubCommand::Set(cmd) => cmd.run(ctx),
            SubCommand::List(cmd) => cmd.run(ctx),
        }
    }
}

/// Delete an alias.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAliasDelete {
    #[clap(name = "alias", required = true)]
    alias: String,
}

impl crate::cmd::Command for CmdAliasDelete {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut alias_config = ctx.config.aliases().unwrap();

        let (expansion, ok) = alias_config.get(&self.alias);
        if !ok {
            bail!("no such alias {}", self.alias);
        }

        match alias_config.delete(&self.alias) {
            Ok(_) => {
                let cs = ctx.io.color_scheme();
                writeln!(
                    ctx.io.err_out,
                    "{} Deleted alias {}; was {}",
                    cs.success_icon_with_color(ansi_term::Color::Red),
                    self.alias,
                    expansion
                )
                .unwrap();
            }
            Err(e) => {
                bail!("failed to delete alias {}: {}", self.alias, e);
            }
        }

        Ok(())
    }
}

/// Create a shortcut for an oxide command.
///
/// Define a word that will expand to a full oxide command when invoked.
///
/// The expansion may specify additional arguments and flags. If the expansion includes
/// positional placeholders such as "$1", extra arguments that follow the alias will be
/// inserted appropriately. Otherwise, extra arguments will be appended to the expanded
/// command.
///
/// Use "-" as expansion argument to read the expansion string from standard input. This
/// is useful to avoid quoting issues when defining expansions.
///
/// If the expansion starts with "!" or if "--shell" was given, the expansion is a shell
/// expression that will be evaluated through the "sh" interpreter when the alias is
/// invoked. This allows for chaining multiple commands via piping and redirection.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAliasSet {
    #[clap(name = "alias", required = true)]
    alias: String,

    #[clap(name = "expansion", required = true)]
    expansion: String,

    /// Declare an alias to be passed through a shell interpreter.
    #[clap(short, long)]
    pub shell: bool,
}

impl crate::cmd::Command for CmdAliasSet {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let cs = ctx.io.color_scheme();

        let mut config_aliases = ctx.config.aliases().unwrap();

        match get_expansion(self) {
            Ok(mut expansion) => {
                let is_terminal = ctx.io.is_stdout_tty();

                if is_terminal {
                    writeln!(
                        ctx.io.err_out,
                        "- Adding alias for {}: %{}",
                        cs.bold(&self.alias),
                        cs.bold(&expansion)
                    )
                    .unwrap();
                }

                let mut is_shell = self.shell;
                if is_shell && !expansion.starts_with('!') {
                    expansion = format!("!{}", expansion);
                }
                is_shell = expansion.starts_with('!');

                // Check if already exists.
                if valid_command(&self.alias) {
                    bail!("could not create alias: {} is already an oxide command", self.alias);
                }

                if !is_shell && valid_command(&expansion) {
                    bail!(
                        "could not create alias: {} does not correspond to an oxide command",
                        expansion
                    );
                }

                let mut success_msg = format!("{} Added alias.", cs.success_icon());
                let (old_expansion, ok) = config_aliases.get(&self.alias);
                if ok {
                    success_msg = format!(
                        "{} Changed alias {} from {} to {}",
                        cs.success_icon(),
                        cs.bold(&self.alias),
                        cs.bold(&old_expansion),
                        cs.bold(&expansion)
                    );
                }

                match config_aliases.add(&self.alias, &expansion) {
                    Ok(_) => {
                        if is_terminal {
                            writeln!(ctx.io.err_out, "{}", success_msg).unwrap();
                        }
                    }
                    Err(e) => {
                        bail!("could not create alias: {}", e);
                    }
                }
            }
            Err(e) => {
                bail!("failed to parse expansion {}: {}", self.expansion, e);
            }
        }

        Ok(())
    }
}

/// List your aliases.
///
/// This command prints out all of the aliases oxide is configured to use.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAliasList {}

impl crate::cmd::Command for CmdAliasList {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let config_aliases = ctx.config.aliases().unwrap();

        if config_aliases.map.is_empty() {
            if ctx.io.is_stdout_tty() {
                writeln!(ctx.io.err_out, "no aliases configured").unwrap();
            }
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        for (alias, expansion) in config_aliases.list().iter() {
            writeln!(tw, "{}:\t{}", alias, expansion).unwrap();
        }
        tw.flush().unwrap();

        let table = String::from_utf8(tw.into_inner().unwrap()).unwrap();
        writeln!(ctx.io.out, "{}", table).unwrap();

        Ok(())
    }
}

fn get_expansion(cmd: &CmdAliasSet) -> Result<String> {
    if cmd.expansion == "-" {
        let mut expansion = String::new();
        std::io::stdin().read_line(&mut expansion)?;
        Ok(expansion)
    } else {
        Ok(cmd.expansion.to_string())
    }
}

fn valid_command(args: &str) -> bool {
    let s = shlex::split(args);
    if s.is_none() {
        return false;
    }

    let split = s.unwrap();

    // Convert our opts into a clap app.
    let app: App = crate::Opts::into_app();

    // Try to get matches.
    app.try_get_matches_from(split).is_ok()
}
