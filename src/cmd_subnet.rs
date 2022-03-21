use std::io::Write;

use anyhow::Result;
use clap::Parser;
use cli_macro::crud_gen;

/// Create, list, edit, view, and delete subnets.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnet {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "subnets",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnet {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Create(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Edit(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::View(cmd) => cmd.run(ctx).await,
        }
    }
}
