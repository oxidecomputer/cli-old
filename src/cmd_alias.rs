use anyhow::Result;
use clap::Parser;

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

impl CmdAlias {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        match &self.subcmd {
            SubCommand::Delete(cmd) => cmd.run(config),
            SubCommand::Set(cmd) => cmd.run(config),
            SubCommand::List(cmd) => cmd.run(config),
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

impl CmdAliasDelete {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        let mut alias_config = config.aliases().unwrap();

        let (expansion, ok) = alias_config.get(&self.alias);
        if !ok {
            eprintln!("no such alias {}", self.alias);
            std::process::exit(1);
        }

        match alias_config.delete(&self.alias) {
            Ok(_) => {
                println!("Deleted alias {}; was {}", self.alias, expansion);
            }
            Err(e) => {
                eprintln!("failed to delete alias {}: {}", self.alias, e);
                std::process::exit(1);
            }
        }
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

    /// Set per-host setting.
    #[clap(short, long)]
    pub shell: bool,
}

impl CmdAliasSet {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        // TODO: check if valid command.

        let mut config_aliases = config.aliases().unwrap();

        match get_expansion(self) {
            Ok(mut expansion) => {
                println!("- Adding alias for {}: %{}", self.alias, expansion);

                let mut is_shell = self.shell;
                if is_shell && !expansion.starts_with("!") {
                    expansion = format!("!{}", expansion);
                }
                is_shell = expansion.starts_with("!");

                // TODO: check if already exists.

                let mut success_msg = format!("Added alias.");
                let (old_expansion, ok) = config_aliases.get(&self.alias);
                if ok {
                    success_msg = format!("Changed alias {} from {} to {}", self.alias, old_expansion, expansion);
                }

                match config_aliases.add(&self.alias, &expansion) {
                    Ok(_) => {
                        println!("{}", success_msg);
                    }
                    Err(e) => {
                        eprintln!("could not create alias: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("failed to parse expansion {}: {}", self.expansion, e);
                std::process::exit(1);
            }
        }
    }
}

/// List your aliases.
///
/// This command prints out all of the aliases oxide is configured to use.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAliasList {}

impl CmdAliasList {
    pub fn run(&self, config: &mut dyn crate::config::Config) {
        let config_aliases = config.aliases().unwrap();

        if config_aliases.map.is_empty() {
            println!("no aliases configured");
            return;
        }

        for (alias, expansion) in config_aliases.list().iter() {
            println!("{}:\t{}", alias, expansion);
        }
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
