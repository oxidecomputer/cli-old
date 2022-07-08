use std::{io::BufRead, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::Parser;
use oxide_api::types::{NameSortMode, SshKeyCreate};
use ssh_key::{
    private::{EcdsaKeypair, Ed25519Keypair, KeypairData, RsaKeypair},
    public::PublicKey,
    rand_core::OsRng,
    Algorithm, EcdsaCurve, LineEnding, PrivateKey,
};

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
    Generate(CmdSSHKeyGenerate),
    List(CmdSSHKeyList),
    SyncFromGithub(CmdSSHKeySyncFromGithub),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKey {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Add(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Generate(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::SyncFromGithub(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Add an SSH key to your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyAdd {
    /// File containing the SSH public key.
    #[clap(required = true)]
    pub public_key_file: PathBuf,

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
        let public_key = PublicKey::read_openssh_file(&self.public_key_file)?;

        let name = if let Some(name) = &self.name {
            name.clone()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("SSH key name")
                .interact_text()?
        };

        let description = if let Some(ref description) = self.description {
            description.clone()
        } else {
            dialoguer::Input::<String>::new()
                .with_prompt("SSH key description")
                .default(public_key.comment().to_string())
                .interact_text()?
        };

        let client = ctx.api_client("")?;
        let params = SshKeyCreate {
            name: name.clone(),
            description,
            public_key: public_key.to_string(),
        };
        client.sshkeys().post(&params).await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Added SSH public key {}: {} {}",
            cs.success_icon(),
            name,
            public_key.algorithm(),
            public_key.fingerprint(Default::default()),
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

/// Generate a new SSH keypair and add the public key to your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyGenerate {
    /// Path to write the SSH private key into.
    /// The public key will be written into this path plus the suffix `.pub`.
    #[clap(required = true)]
    pub private_key_file: PathBuf,

    /// SSH key type to generate.
    #[clap(long = "type", short = 't', default_value = "ed25519", parse(try_from_str = parse_algorithm))]
    pub key_type: Algorithm,

    /// Number of bits in the key to generate.
    #[clap(long = "bits", short = 'b')]
    pub key_size: Option<usize>,

    /// Comment for the SSH key.
    #[clap(long, short, default_value_t)]
    pub comment: String,

    /// The name of the SSH key.
    #[clap(long, short)]
    pub name: Option<String>,

    /// Description of the SSH key.
    #[clap(long, short = 'D')]
    pub description: Option<String>,
}

fn parse_algorithm(algorithm: &str) -> Result<Algorithm> {
    match algorithm.to_lowercase().as_str() {
        "ecdsa" => Ok(Algorithm::Ecdsa {
            curve: EcdsaCurve::NistP256, // may be overridden by key size
        }),
        "ed25519" => Ok(Algorithm::Ed25519),
        "rsa" => Ok(Algorithm::Rsa {
            hash: Default::default(),
        }),
        _ => Err(anyhow!("supported types are `ecdsa`, `ed25512`, and `rsa`")),
    }
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyGenerate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let private_key = match self.key_type {
            Algorithm::Ecdsa { mut curve } => {
                // Note that ssh_key can currently only generate P256 keys
                if let Some(bits) = self.key_size {
                    curve = match bits {
                        256 => EcdsaCurve::NistP256,
                        //384 => EcdsaCurve::NistP384,
                        //521 => EcdsaCurve::NistP521,
                        _ => return Err(anyhow!("ECDSA key length must be 256")),
                    };
                }
                let keypair = EcdsaKeypair::random(OsRng, curve)?;
                PrivateKey::new(KeypairData::Ecdsa(keypair), &self.comment)?
            }
            Algorithm::Ed25519 => {
                // Ed255129 keys are always fixed length, so ignore key_size
                let keypair = Ed25519Keypair::random(OsRng);
                PrivateKey::new(KeypairData::Ed25519(keypair), &self.comment)?
            }
            Algorithm::Rsa { .. } => {
                // Generating large RSA keys can be quite slow, so use a spinner
                let bits = self.key_size.unwrap_or(3072);
                let spinner = ctx
                    .io
                    .start_process_indicator_with_label(&format!(" Generating {} bit RSA key", bits));
                let keypair = RsaKeypair::random(OsRng, bits)?;
                spinner.map(|spinner| spinner.stop());
                PrivateKey::new(KeypairData::Rsa(keypair), &self.comment)?
            }
            _ => unimplemented!("generate a random {} key", self.key_type),
        };
        private_key.write_openssh_file(&self.private_key_file, LineEnding::default())?;

        let public_key = private_key.public_key();
        let mut public_key_file = self.private_key_file.clone();
        public_key_file.set_extension("pub");
        public_key.write_openssh_file(&public_key_file)?;

        let name = self.name.clone();
        let description = self.description.clone();
        CmdSSHKeyAdd {
            public_key_file,
            name,
            description,
        }
        .run(ctx)
        .await
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
            client
                .sshkeys()
                .get_page(self.limit, "", NameSortMode::NameAscending)
                .await?
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

/// Retrieve the public SSH keys for a specific github user.
async fn get_github_ssh_keys(gh_handle: &str) -> Result<Vec<PublicKey>> {
    let resp = reqwest::get(&format!("https://github.com/{}.keys", gh_handle)).await?;
    let body = resp.bytes().await?;

    let reader = std::io::BufReader::new(body.as_ref());
    let lines: Vec<_> = reader.lines().collect();

    let mut keys: Vec<PublicKey> = Vec::new();
    for l in lines {
        let line = l?;
        // Parse the key.
        let key = PublicKey::from_openssh(&line)?;

        // Add the key to the list.
        keys.push(key);
    }

    Ok(keys)
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
            let comment = if key.comment().is_empty() {
                format!("From GitHub user {}", self.github_username)
            } else {
                key.comment().to_string()
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
                key.algorithm(),
                key.fingerprint(Default::default()),
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

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn test_get_github_ssh_keys() {
        let result = super::get_github_ssh_keys("jessfraz").await;
        assert!(!result.expect("failed to get keys from GitHub").is_empty());
    }
}
