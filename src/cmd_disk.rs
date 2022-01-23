use std::io::Write;

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
pub struct CmdDiskList {
    /// The project that holds the disks.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Maximum number of disks to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of disks.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        let disks = if self.paginate {
            client
                .disks()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                )
                .await?
        } else {
            client
                .disks()
                .get_page(
                    self.limit,
                    "",
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                )
                .await?
        };

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(disks))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tSTATE\tDEVICE PATH\tLAST UPDATED")?;
        for disk in disks {
            let last_updated = chrono::Utc::now() - disk.time_modified.unwrap_or_else(|| disk.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{:?}\t{}\t{}",
                &disk.name,
                &disk.description,
                &disk.state,
                &disk.device_path,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

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
pub struct CmdDiskView {
    /// The disk to view.
    #[clap(name = "disk", required = true)]
    pub disk: String,

    /// The project that holds the disk.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization to view the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Open a project in the browser.
    #[clap(short, long)]
    pub web: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}/disks/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project,
                self.disk
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let disk = client
            .disks()
            .get(&self.disk, &self.organization, &self.project)
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(disk))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", disk.id)?;
        writeln!(tw, "name:\t{}", disk.name)?;
        writeln!(tw, "description:\t{}", disk.description)?;
        // TODO: implement display for these enums in the api lib.
        writeln!(tw, "state:\t{:?}", disk.state)?;
        writeln!(tw, "size:\t{}", disk.size)?;
        writeln!(tw, "device path:\t{}", disk.device_path)?;
        writeln!(tw, "snapshot:\t{}", disk.snapshot_id)?;
        if let Some(time_created) = disk.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = disk.time_modified {
            writeln!(
                tw,
                "modified:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_modified)
            )?;
        }

        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}
