use std::collections::HashMap;

use anyhow::{anyhow, Result};
use clap::Parser;

/// Login, logout, and get the status of your authentication.
///
/// Manage `oxide`'s authentication state.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAuth {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Login(CmdAuthLogin),
    Logout(CmdAuthLogout),
    Status(CmdAuthStatus),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdAuth {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Login(cmd) => cmd.run(ctx).await,
            SubCommand::Logout(cmd) => cmd.run(ctx).await,
            SubCommand::Status(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Authenticate with an Oxide host.
///
/// Alternatively, pass in a token on standard input by using `--with-token`.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAuthLogin {
    /// Read token from standard input.
    #[clap(long)]
    pub with_token: bool,

    /// The hostname of the Oxide instance to authenticate with.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
    // Open a browser to authenticate.
    // TODO: Make this work when we have device auth.
    // #[clap(short, long)]
    // pub web: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdAuthLogin {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Log out of an Oxide host.
///
/// This command removes the authentication configuration for a host either specified
/// interactively or via `--host`.
///
///     $ oxide auth logout
///     # => select what host to log out of via a prompt
///
///     $ oxide auth logout --host oxide.internal
///     # => log out of specified host
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAuthLogout {
    /// The hostname of the Oxide instance to log out of.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdAuthLogout {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.host.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--host required when not running interactively"));
        }

        let mut hostname = self.host.to_string();

        let candidates = ctx.config.hosts()?;
        if candidates.is_empty() {
            return Err(anyhow!("not logged in to any hosts"));
        }

        if hostname.is_empty() {
            if candidates.len() == 1 {
                hostname = candidates[0].to_string();
            } else {
                let index = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
                    .with_prompt("What account do you want to log out of?")
                    .default(0)
                    .items(&candidates[..])
                    .interact();

                match index {
                    Ok(i) => {
                        hostname = candidates[i].to_string();
                    }
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }
        } else {
            let mut found = false;
            for c in candidates {
                if c == hostname {
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(anyhow!("not logged into {}", hostname));
            }
        }

        if let Err(err) = ctx.config.check_writable(&hostname, "token") {
            // TODO: make this error a type and actually check it.
            if let Some(crate::config_from_env::ReadOnlyEnvVarError::Variable(var)) = err.downcast_ref() {
                writeln!(
                    ctx.io.err_out,
                    "The value of the {} environment variable is being used for authentication.\n",
                    var
                )?;
                writeln!(
                    ctx.io.err_out,
                    "To erase credentials stored in Oxide CLI, first clear the value from the environment.\n"
                )?;
                return Err(anyhow!(""));
            }

            return Err(err);
        }

        let _client = ctx.api_client(&hostname)?;

        // TODO: Get the current user.
        // let session = client.session().await?;

        /*let username_str = if session.email.is_empty() {
            "".to_string()
        } else {
            format!(" account '{}'", session.email)
        };*/
        let username_str = "".to_string();

        if ctx.io.can_prompt() {
            /*	var keepGoing bool
            err := prompt.SurveyAskOne(&survey.Confirm{
                Message: fmt.Sprintf("Are you sure you want to log out of %s%s?", hostname, usernameStr),
                Default: true,
            }, &keepGoing)
            if err != nil {
                return fmt.Errorf("could not prompt: %w", err)
            }

            if !keepGoing {
                return nil
            }*/
        }

        // Unset the host.
        ctx.config.unset_host(&hostname)?;

        // Write the changes to the config.
        ctx.config.write()?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Logged out of {}{}\n",
            cs.success_icon(),
            cs.bold(&hostname),
            username_str
        )?;

        Ok(())
    }
}

/// Verifies and displays information about your authentication state.
///
/// This command will test your authentication state for each Oxide host that `oxide`
/// knows about and report on any issues.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAuthStatus {
    /// Display the auth token.
    #[clap(short = 't', long)]
    pub show_token: bool,

    /// Check a specific hostname's auth status.
    #[clap(short = 'H', long, default_value = "")]
    pub host: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdAuthStatus {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let cs = ctx.io.color_scheme();

        let mut status_info: HashMap<String, Vec<String>> = HashMap::new();

        let hostnames = ctx.config.hosts()?;

        if hostnames.is_empty() {
            writeln!(
                ctx.io.err_out,
                "You are not logged into any Oxide hosts. Run {} to authenticate.\n",
                cs.bold("oxide auth login")
            )?;
            return Err(anyhow!(""));
        }

        let mut failed = false;
        let mut hostname_found = false;

        for hostname in &hostnames {
            if !self.host.is_empty() && self.host != *hostname {
                continue;
            }

            hostname_found = true;

            let (token, token_source) = ctx.config.get_with_source(hostname, "token")?;

            let client = ctx.api_client(hostname)?;

            let mut host_status: Vec<String> = vec![];

            /*match client.session().await {
                Ok(session) => {
                    // Let the user know if their token is invalid.
                    if !session.is_valid() {
                        host_status.push(format!(
                            "{} Logged in to {} as {} ({}) with an invalid token",
                            cs.failure_icon(),
                            hostname,
                            cs.bold(session.email),
                            token_source
                        ));
                        failed = true;
                        continue;
                    }

                    host_status.push(format!(
                        "{} Logged in to {} as {} ({})",
                        cs.success_icon(),
                        hostname,
                        cs.bold(session.email),
                        token_source
                    ));
                    let mut token_display = "*******************".to_string();
                    if self.show_token {
                        token_display = token.to_string();
                    }
                    host_status.push(format!("{} Token: {}\n", cs.success_icon(), token_display));
                }
                Err(err) => {
                    host_status.push(format!("{} {}: api call failed: {}", cs.failure_icon(), hostname, err));
                    failed = true;
                    continue;
                }
            }*/

            status_info.insert(hostname.to_string(), host_status);
        }

        if !hostname_found {
            writeln!(
                ctx.io.err_out,
                "Hostname {} not found among authenticated Oxide hosts\n",
                self.host
            )?;
            return Err(anyhow!(""));
        }

        for hostname in hostnames {
            match status_info.get(&hostname) {
                Some(status) => {
                    writeln!(ctx.io.out, "{}", cs.bold(&hostname))?;
                    for line in status {
                        writeln!(ctx.io.out, "{}", line)?;
                    }
                }
                None => {
                    writeln!(ctx.io.err_out, "No status information for {}\n", hostname)?;
                }
            }
        }

        if failed {
            return Err(anyhow!(""));
        }

        Ok(())
    }
}
