use anyhow::Result;
use clap::Parser;

/// Create, list, edit, view, and delete instances.
///
/// Additionally, start, stop, and reboot instances.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstance {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdInstanceCreate),
    Delete(CmdInstanceDelete),
    Edit(CmdInstanceEdit),
    List(CmdInstanceList),
    Start(CmdInstanceStart),
    Stop(CmdInstanceStop),
    Reboot(CmdInstanceReboot),
    View(CmdInstanceView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstance {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Create(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Edit(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::Start(cmd) => cmd.run(ctx).await,
            SubCommand::Stop(cmd) => cmd.run(ctx).await,
            SubCommand::Reboot(cmd) => cmd.run(ctx).await,
            SubCommand::View(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Create a new instance.
///
/// To create an instance interactively, use `oxide instance create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceCreate {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceCreate {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Delete an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceDelete {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceDelete {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Edit instance settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// List instances owned by a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceList {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceList {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Start an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStart {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStart {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Stop an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStop {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStop {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// Reboot an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceReboot {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceReboot {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

/// View an instance.
///
/// Display the description and other information of an Oxide instance.
///
/// With '--web', open the instance in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceView {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceView {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}
