use anyhow::Result;
use clap::Parser;

/// Manage ssh keys.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKey {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Add(CmdSSHKeyAdd),
    Delete(CmdSSHKeyDelete),
    List(CmdSSHKeyList),
    SyncFromGithub(CmdSSHKeySyncFromGithub),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKey {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Add(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::SyncFromGithub(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Add an ssh key to your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyAdd {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyAdd {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        // Let's generate a key.
        let key = crate::ssh::SSHKeyPair::generate(&crate::ssh::SSHKeyAlgorithm::Ed25519)?;

        writeln!(ctx.io.out, "Your SSH key is: {:?}", key)?;

        let pubkey = key.public_key()?;

        writeln!(ctx.io.out, "Your SSH public key is: {:?}", pubkey)?;

        Ok(())
    }
}

/// Delete an ssh key from your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyDelete {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyDelete {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        todo!()
    }
}

/// List ssh keys in your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyList {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyList {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        todo!()
    }
}

/// Sync your public ssh keys from GitHub to your Oxide account.
///
/// This command will retrieve your public ssh keys from GitHub and add them
/// to your Oxide account.
///
/// You will not need to authenticate with GitHub as your public ssh keys are
/// public information.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeySyncFromGithub {
    /// Your GitHub username.
    #[clap(name = "github_username", required = true)]
    pub github_username: String,

    /// Remove any keys from your Oxide account that are not in your GitHub account.
    /// This is useful if you want to use your GitHub account as the ultimate source
    /// of your ssh keys.
    #[clap(long = "overwrite")]
    pub remove_unsynced_keys: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeySyncFromGithub {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let cs = ctx.io.color_scheme();

        let keys = crate::ssh::get_github_ssh_keys(&self.github_username).await?;

        for key in keys {
            // TODO: add the key to Oxide.
            writeln!(
                ctx.io.out,
                "{} Added key `{} {}`\n\t`{}`",
                cs.success_icon(),
                key.key_type.name,
                key.fingerprint(),
                key,
            )?;

            // TODO: print if a key already exists.
        }

        // TODO: make the overwrite flag work.

        writeln!(
            ctx.io.out,
            "{} Oxide ssh keys synced with GitHub user `{}`!",
            cs.success_icon(),
            self.github_username
        )?;

        Ok(())
    }
}
