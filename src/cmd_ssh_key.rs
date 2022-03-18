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
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKey {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Add(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Add an ssh key to your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyAdd {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyAdd {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
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
        println!("Not implemented yet.");
        Ok(())
    }
}

/// List ssh keys in your Oxide account.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSSHKeyList {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSSHKeyList {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}
