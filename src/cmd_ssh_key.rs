use crate::ssh::{get_default_ssh_key, get_github_ssh_keys, SSHKeyAlgorithm};

use oxide_api::types::{NameSortMode, SshKeyCreate};

use anyhow::{anyhow, Result};
use clap::Parser;
use sshkeys::PublicKey;
use std::path::PathBuf;

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

    /// SSH key type to generate.
    #[clap(long = "type", short = 't', default_value_t)]
    pub key_type: SSHKeyAlgorithm,

    /// File containing the SSH public key.
    #[clap(long, short)]
    pub file: Option<PathBuf>,

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
        let (pubkey, path) = if self.generate {
            todo!("generate a key pair and write both halves to files");
            //let keypair = SSHKeyPair::generate(&self.algorithm)?;
            //writeln!(ctx.io.out, "Your SSH key pair is: {:?}", keypair)?;
            //keypair.public_key()?
        } else if let Some(ref path) = self.file {
            (PublicKey::from_path(path)?, path.clone())
        } else {
            get_default_ssh_key(&self.key_type)?
        };
        writeln!(ctx.io.out, "Read SSH public key from {}", path.display())?;

        let name = if let Some(name) = &self.name {
            name.clone()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("SSH key name")
                .interact_text()?
        };

        let comment = match pubkey.comment {
            Some(ref c) => c,
            None => "",
        };

        let description = if let Some(ref description) = self.description {
            description.clone()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("SSH key description")
                .default(comment.to_string())
                .interact_text()?
        };

        let client = ctx.api_client("")?;
        let params = SshKeyCreate {
            name: name.clone(),
            description,
            public_key: pubkey.to_string(),
        };
        client.sshkeys().post(&params).await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Added SSH public key {}: {} {}",
            cs.success_icon(),
            name,
            pubkey.key_type,
            pubkey.fingerprint(),
        )?;

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

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted SSH key {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.name
        )?;

        Ok(())
    }
}

/// List SSH keys in your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyList {
    /// Maximum number of SSH keys to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages.
    #[clap(long)]
    pub paginate: bool,

    /// Output format.
    #[clap(long, short)]
    pub format: Option<crate::types::FormatOutput>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;
        let results = if self.paginate {
            client.sshkeys().get_all(NameSortMode::NameAscending).await?
        } else {
            client.sshkeys().get_page(self.limit, "", NameSortMode::NameAscending).await?
        };

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

        if self.remove_unsynced_keys {
            todo!("make the overwrite flag work");
        }

        let keys = get_github_ssh_keys(&self.github_username).await?;
        let names = match keys.len() {
            0 => vec![],
            1 => vec![self.github_username.clone()],
            _ => keys
                .iter()
                .enumerate()
                .map(|(i, _key)| format!("{}-{}", self.github_username, i))
                .collect::<Vec<String>>(),
        };

        let client = ctx.api_client("")?;
        for (key, name) in keys.into_iter().zip(names) {
            let comment = match key.comment {
                Some(ref c) => c.clone(),
                None => format!("From GitHub user {}", self.github_username),
            };
            let params = SshKeyCreate {
                name: name.clone(),
                description: comment,
                public_key: key.to_string(),
            };

            // TODO: warn if a key already exists.
            client.sshkeys().post(&params).await?;

            writeln!(
                ctx.io.out,
                "{} Added SSH public key {}: {} {}",
                cs.success_icon(),
                name,
                key.key_type,
                key.fingerprint(),
            )?;
        }

        writeln!(
            ctx.io.out,
            "{} Oxide SSH keys synced with GitHub user {}!",
            cs.success_icon(),
            self.github_username
        )?;

        Ok(())
    }
}
