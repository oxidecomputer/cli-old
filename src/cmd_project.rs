use anyhow::Result;
use clap::Parser;

/// Create, list, edit, view, and delete projects.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProject {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdProjectCreate),
    Delete(CmdProjectDelete),
    Edit(CmdProjectEdit),
    List(CmdProjectList),
    View(CmdProjectView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProject {
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

/// Create a new project.
///
/// To create a project interactively, use `oxide project create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectCreate {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectCreate {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Delete a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectDelete {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectDelete {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Edit project settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// List projects owned by user or organization.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectList {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectList {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// View a project.
///
/// Display the description and other information of an Oxide project.
///
/// With '--web', open the project in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectView {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectView {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}
