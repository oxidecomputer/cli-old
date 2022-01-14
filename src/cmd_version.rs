use anyhow::Result;
use clap::Parser;

/// Prints the version of the program.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVersion {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdVersion {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let version = clap::crate_version!();

        writeln!(ctx.io.out, "oxide {}", version)?;

        writeln!(ctx.io.out, "{}", changelog_url(version))?;

        Ok(())
    }
}

fn changelog_url(version: &str) -> String {
    format!("https://github.com/oxidecomputer/cli/releases/tag/v{}", version)
}