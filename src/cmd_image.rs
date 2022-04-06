use anyhow::Result;
use clap::Parser;
use cli_macro::crud_gen;

/// Create, list, view, and delete images.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdImage {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "images",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdImage {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        todo!()
    }
}
