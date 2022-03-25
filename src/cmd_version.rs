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
        let git_hash = git_rev::try_revision_string!();

        if let Some(gh) = git_hash {
            writeln!(ctx.io.out, "oxide {} ({})", version, gh);
        } else {
            writeln!(ctx.io.out, "oxide {}", version);
        }

        writeln!(ctx.io.out, "{}", changelog_url(version))?;

        Ok(())
    }
}

/// Returns the URL to the changelog for the given version.
pub fn changelog_url(version: &str) -> String {
    format!("https://github.com/oxidecomputer/cli/releases/tag/v{}", version)
}
