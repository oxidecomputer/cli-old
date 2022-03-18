use anyhow::Result;
use clap::Parser;
use cli_macros::crud_gen;

/// Manage racks.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRack {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "racks",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRack {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            _ => todo!(),
        }
    }
}
