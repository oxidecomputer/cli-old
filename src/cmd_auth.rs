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
///
///     # start interactive setup
///     $ oxide auth login
///
///     # authenticate against a specific Oxide instance by reading the token from a file
///     $ oxide auth login --with-token --host oxide.internal < mytoken.txt
///
///     # authenticate with a specific Oxide instance
///     $ oxide auth login --host oxide.internal
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdAuthLogin {
    /// Read token from standard input.
    #[clap(long)]
    pub with_token: bool,

    /// The hostname of the Oxide instance to authenticate with.
    #[clap(short = 'H', long, env = "OXIDE_HOST", default_value = "")]
    pub host: String,
    // Open a browser to authenticate.
    // TODO: Make this work when we have device auth.
    // #[clap(short, long)]
    // pub web: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdAuthLogin {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.with_token {
            return Err(anyhow!("--with-token required when not running interactively"));
        }

        let mut token = String::new();

        if self.with_token {
            // Read from stdin.
            ctx.io.stdin.read_to_string(&mut token)?;
        }

        let mut interactive = false;
        if ctx.io.can_prompt() && token.is_empty() {
            interactive = true;
        }

        let mut host = clean_hostname(&self.host);

        if host.is_empty() {
            if interactive {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Oxide instance hostname:")
                    .interact_text()
                {
                    Ok(input) => {
                        host = clean_hostname(&input);
                    }
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            } else {
                return Err(anyhow!("--host required when not running interactively"));
            }
        }

        if let Err(err) = ctx.config.check_writable(&host, "token") {
            if let Some(crate::config_from_env::ReadOnlyEnvVarError::Variable(var)) = err.downcast_ref() {
                writeln!(
                    ctx.io.err_out,
                    "The value of the {} environment variable is being used for authentication.",
                    var
                )?;
                writeln!(
                    ctx.io.err_out,
                    "To have Oxide CLI store credentials instead, first clear the value from the environment."
                )?;
                return Err(anyhow!(""));
            }

            return Err(err);
        }

        let cs = ctx.io.color_scheme();

        // Do the login flow if we didn't get a token from stdin.
        if token.is_empty() {
            // We don't want to capture the error here just in case we have no host config
            // for this specific host yet.
            let existing_token = if let Ok(existing_token) = ctx.config.get(&host, "token") {
                existing_token
            } else {
                String::new()
            };
            if !existing_token.is_empty() && interactive {
                match dialoguer::Confirm::new()
                    .with_prompt(format!(
                        "You're already logged into {}. Do you want to re-authenticate?",
                        host
                    ))
                    .interact()
                {
                    Ok(true) => {}
                    Ok(false) => {
                        return Ok(());
                    }
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            writeln!(
                ctx.io.err_out,
                "Tip: you can generate an API Token here https://{}/account",
                host
            )?;

            token = match dialoguer::Input::<String>::new()
                .with_prompt("Paste your authentication token:")
                .interact_text()
            {
                Ok(input) => input,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            };
        }

        // Set the token in the config file.
        ctx.config.set(&host, "token", &token)?;

        let client = ctx.api_client(&host)?;

        // Get the session for the token.
        let session = client.hidden().session_me().await?;

        // Set the user.
        // TODO: This should instead store the email, or some username or something
        // that is human knowable.
        let email = session.id;
        ctx.config.set(&host, "user", &email)?;

        // Save the config.
        ctx.config.write()?;

        writeln!(ctx.io.out, "{} Logged in as {}", cs.success_icon(), cs.bold(&email))?;

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
    #[clap(short = 'H', long, default_value = "", env = "OXIDE_HOST")]
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
            if let Some(crate::config_from_env::ReadOnlyEnvVarError::Variable(var)) = err.downcast_ref() {
                writeln!(
                    ctx.io.err_out,
                    "The value of the {} environment variable is being used for authentication.",
                    var
                )?;
                writeln!(
                    ctx.io.err_out,
                    "To erase credentials stored in Oxide CLI, first clear the value from the environment."
                )?;
                return Err(anyhow!(""));
            }

            return Err(err);
        }

        let client = ctx.api_client(&hostname)?;

        // TODO: Get the current user.
        let session = client.hidden().session_me().await?;

        // TODO: this should be the users email or something better.
        // make it consistent with login.
        let email = session.id;

        if ctx.io.can_prompt() {
            match dialoguer::Confirm::new()
                .with_prompt(format!("Are you sure you want to log out of {}{}?", hostname, email))
                .interact()
            {
                Ok(true) => {}
                Ok(false) => {
                    return Ok(());
                }
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }
        }

        // Unset the host.
        ctx.config.unset_host(&hostname)?;

        // Write the changes to the config.
        ctx.config.write()?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Logged out of {} {}",
            cs.success_icon(),
            cs.bold(&hostname),
            email
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
                ctx.io.out,
                "You are not logged into any Oxide hosts. Run {} to authenticate.",
                cs.bold("oxide auth login")
            )?;
            return Ok(());
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

            match client.hidden().session_me().await {
                Ok(session) => {
                    // TODO: this should be the users email or something consistent with login
                    // and logout.
                    let email = session.id.to_string();
                    // Let the user know if their token is invalid.
                    /*if !session.is_valid() {
                    host_status.push(format!(
                        "{} Logged in to {} as {} ({}) with an invalid token",
                        cs.failure_icon(),
                        hostname,
                        cs.bold(&email),
                        token_source
                    ));
                    failed = true;
                    continue;
                    }*/

                    host_status.push(format!(
                        "{} Logged in to {} as {} ({})",
                        cs.success_icon(),
                        hostname,
                        cs.bold(&email),
                        token_source
                    ));
                    let mut token_display = "*******************".to_string();
                    if self.show_token {
                        token_display = token.to_string();
                    }
                    host_status.push(format!("{} Token: {}", cs.success_icon(), token_display));
                }
                Err(err) => {
                    host_status.push(format!("{} {}: api call failed: {}", cs.failure_icon(), hostname, err));
                    failed = true;
                    continue;
                }
            }

            status_info.insert(hostname.to_string(), host_status);
        }

        if !hostname_found {
            writeln!(
                ctx.io.err_out,
                "Hostname {} not found among authenticated Oxide hosts",
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
                    writeln!(ctx.io.err_out, "No status information for {}", hostname)?;
                }
            }
        }

        if failed {
            return Err(anyhow!(""));
        }

        Ok(())
    }
}

fn clean_hostname(host: &str) -> String {
    host.trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string()
}

#[cfg(test)]
mod test {
    //use pretty_assertions::assert_eq;

    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        cmd: crate::cmd_auth::SubCommand,
        stdin: String,
        want_out: String,
        want_err: String,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cmd_auth() {
        let test_host = std::env::var("OXIDE_TEST_HOST").unwrap_or_default().to_string();
        let test_token = std::env::var("OXIDE_TEST_TOKEN").unwrap_or_default().to_string();

        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "status".to_string(),
                cmd: crate::cmd_auth::SubCommand::Status(crate::cmd_auth::CmdAuthStatus {
                    show_token: false,
                    host: "".to_string(),
                }),
                stdin: "".to_string(),
                want_out: "You are not logged into any Oxide hosts. Run oxide auth login to authenticate.\n"
                    .to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "login --with-token=false".to_string(),
                cmd: crate::cmd_auth::SubCommand::Login(crate::cmd_auth::CmdAuthLogin {
                    host: test_host.to_string(),
                    with_token: false,
                }),
                stdin: test_token.to_string(),
                want_out: "".to_string(),
                want_err: "--with-token required when not running interactively".to_string(),
            },
            TestItem {
                name: "login --with-token=true".to_string(),
                cmd: crate::cmd_auth::SubCommand::Login(crate::cmd_auth::CmdAuthLogin {
                    host: test_host.to_string(),
                    with_token: true,
                }),
                stdin: test_token.to_string(),
                want_out: "✔ Logged in as ".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "status".to_string(),
                cmd: crate::cmd_auth::SubCommand::Status(crate::cmd_auth::CmdAuthStatus {
                    show_token: false,
                    host: "".to_string(),
                }),
                stdin: "".to_string(),
                want_out: format!("{}\n✔ Logged in to {} as", test_host, test_host),
                want_err: "".to_string(),
            },
            TestItem {
                name: "logout no prompt no host".to_string(),
                cmd: crate::cmd_auth::SubCommand::Logout(crate::cmd_auth::CmdAuthLogout { host: "".to_string() }),
                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--host required when not running interactively".to_string(),
            },
            TestItem {
                name: "logout no prompt with host".to_string(),
                cmd: crate::cmd_auth::SubCommand::Logout(crate::cmd_auth::CmdAuthLogout {
                    host: test_host.to_string(),
                }),
                stdin: "".to_string(),
                want_out: "✔ Logged out of localhost:8888 ".to_string(),
                want_err: "".to_string(),
            },
        ];

        let mut config = crate::config::new_blank_config().unwrap();
        let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

        for t in tests {
            let (mut io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
            if !t.stdin.is_empty() {
                io.stdin = Box::new(std::io::Cursor::new(t.stdin));
            }
            // We need to also turn off the fancy terminal colors.
            // This ensures it also works in GitHub actions/any CI.
            io.set_color_enabled(false);
            // TODO: we should figure out how to test the prompts.
            io.set_never_prompt(true);
            let mut ctx = crate::context::Context {
                config: &mut c,
                io,
                debug: false,
            };

            let cmd_auth = crate::cmd_auth::CmdAuth { subcmd: t.cmd };
            match cmd_auth.run(&mut ctx).await {
                Ok(()) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert!(stderr.is_empty(), "test {}: {}", t.name, stderr);
                    if !stdout.contains(&t.want_out) {
                        assert_eq!(stdout, t.want_out, "test {}: stdout mismatch", t.name);
                    }
                }
                Err(err) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert_eq!(stdout, t.want_out, "test {}", t.name);
                    if !err.to_string().contains(&t.want_err) {
                        assert_eq!(err.to_string(), t.want_err, "test {}: err mismatch", t.name);
                    }
                    assert!(stderr.is_empty(), "test {}: {}", t.name, stderr);
                }
            }
        }
    }
}
