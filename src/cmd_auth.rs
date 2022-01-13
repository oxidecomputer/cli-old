use anyhow::Result;
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
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::cmd::Command;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_auth() {}
}
