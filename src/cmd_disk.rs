use anyhow::{anyhow, Result};
use clap::Parser;

/// Create, list, edit, view, and delete disks.
///
/// Additionally, attach and detach disks to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDisk {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Attach(CmdDiskAttach),
    Create(CmdDiskCreate),
    Delete(CmdDiskDelete),
    Detach(CmdDiskDetach),
    Edit(CmdDiskEdit),
    List(CmdDiskList),
    View(CmdDiskView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDisk {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Attach(cmd) => cmd.run(ctx).await,
            SubCommand::Create(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Detach(cmd) => cmd.run(ctx).await,
            SubCommand::Edit(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::View(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Attach a disk to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskAttach {
    /// The disk to attach. Can be an ID or name.
    #[clap(name = "disk", required = true)]
    disk: String,

    /// The instance to attach the disk to. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the disk and instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskAttach {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Attach the disk.
        client
            .instances()
            .disks_attach(
                &self.instance,
                &self.organization,
                &self.project,
                &oxide_api::types::DiskIdentifier {
                    disk: self.disk.to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Attached disk {} to instance {} in project {}",
            cs.success_icon(),
            self.disk,
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Create a new disk.
///
/// To create a disk interactively, use `oxide disk create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskCreate {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskCreate {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// Delete a disk.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskDelete {
    /// The disk to delete. Can be an ID or name.
    #[clap(name = "disk", required = true)]
    disk: String,

    /// The project to delete the disk from.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm deletion without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm deletion.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.disk))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == full_name {
                        Ok(())
                    } else {
                        Err("mismatched confirmation")
                    }
                })
                .interact_text()
            {
                return Err(anyhow!("prompt failed: {}", err));
            }
        }

        // Delete the project.
        client
            .disks()
            .delete(&self.disk, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted disk {} from {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.disk,
            full_name
        )?;

        Ok(())
    }
}

/// Detach a disk from an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskDetach {
    /// The disk to detach. Can be an ID or name.
    #[clap(name = "disk", required = true)]
    disk: String,

    /// The instance to detach the disk from. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the disk and instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskDetach {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Detach the disk.
        client
            .instances()
            .disks_detach(
                &self.instance,
                &self.organization,
                &self.project,
                &oxide_api::types::DiskIdentifier {
                    disk: self.disk.to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Detached disk {} from instance {} in project {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.disk,
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Edit disk settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// List disks owned by a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskList {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskList {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// View a disk.
///
/// Display the description and other information of an Oxide disk.
///
/// With '--web', open the disk in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskView {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskView {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}
