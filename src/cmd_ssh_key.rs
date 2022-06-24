use crate::ssh::{get_default_ssh_key, get_github_ssh_keys, SSHKeyAlgorithm, SSHKeyPair};

use oxide_api::types::{NameSortMode, SshKeyCreate};

use anyhow::Result;
use clap::Parser;

/// Manage SSH keys.
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

/// Add an SSH key to your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyAdd {
    /// Generate a new SSH key.
    #[clap(long, short)]
    pub generate: bool,

    /// SSH key type to use.
    #[clap(long, short, default_value_t)]
    pub algorithm: SSHKeyAlgorithm,

    /// The name of the SSH key.
    #[clap(long, short)]
    pub name: Option<String>,

    /// Description of the SSH key.
    #[clap(long, short = 'D')]
    pub description: Option<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyAdd {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let pubkey = if self.generate {
            todo!("generate a key pair and write both halves to files");
            //let keypair = SSHKeyPair::generate(&self.algorithm)?;
            //writeln!(ctx.io.out, "Your SSH key pair is: {:?}", keypair)?;
            //keypair.public_key()?
        } else {
            get_default_ssh_key(&self.algorithm)?
        };

        let name = if let Some(name) = &self.name {
            name.clone()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("SSH key name")
                .interact_text()?
        };

        let description = if let Some(description) = &self.description {
            description.clone()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("SSH key description")
                .default(
                    pubkey
                        .comment
                        .as_ref()
                        .map(|c| c.clone())
                        .unwrap_or_else(|| "".to_string()),
                )
                .interact_text()?
        };

        let client = ctx.api_client("")?;
        let params = SshKeyCreate {
            name,
            description,
            public_key: pubkey.to_string(),
        };
        client.sshkeys().post(&params).await?;

        writeln!(ctx.io.out, "Added SSH public key: {}", pubkey)?;
        Ok(())
    }
}

/// Delete an SSH key from your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyDelete {
    /// The name of the SSH key to delete.
    #[clap(required = true)]
    pub name: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;
        client.sshkeys().delete_key(&self.name).await?;
        writeln!(ctx.io.out, "Deleted SSH key {}", self.name)?;
        Ok(())
    }
}

/// List SSH keys in your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyList {
    /// Output format.
    #[clap(long, short)]
    pub format: Option<crate::types::FormatOutput>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyList {
    // TODO: support pagination
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;
        let results = client.sshkeys().get_all(NameSortMode::NameAscending).await?;
        let format = ctx.format(&self.format)?;
        ctx.io.write_output_for_vec(&format, &results)?;
        Ok(())
    }
}

/// Sync your public SSH keys from GitHub to your Oxide account.
///
/// This command will retrieve your public SSH keys from GitHub and add them
/// to your Oxide account.
///
/// You will not need to authenticate with GitHub as your public SSH keys are
/// public information.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeySyncFromGithub {
    /// Your GitHub username.
    #[clap(name = "github_username", required = true)]
    pub github_username: String,

    /// Remove any keys from your Oxide account that are not in your GitHub account.
    /// This is useful if you want to use your GitHub account as the ultimate source
    /// of your SSH keys.
    #[clap(long = "overwrite")]
    pub remove_unsynced_keys: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeySyncFromGithub {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let cs = ctx.io.color_scheme();

        let keys = get_github_ssh_keys(&self.github_username).await?;

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
            "{} Oxide SSH keys synced with GitHub user `{}`!",
            cs.success_icon(),
            self.github_username
        )?;

        Ok(())
    }
}
