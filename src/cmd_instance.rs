use std::io::Write;

use anyhow::{anyhow, Result};
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
    Disks(CmdInstanceDisks),
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
            SubCommand::Disks(cmd) => cmd.run(ctx).await,
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
        println!("Not implemented yet.");
        Ok(())
    }
}

/// Delete an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceDelete {
    /// The instance to delete. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project to delete the instance from.
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
impl crate::cmd::Command for CmdInstanceDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm deletion.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.instance))
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

        // Delete the instance.
        client
            .instances()
            .delete(&self.instance, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted instance {} from {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// List the disks attached to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceDisks {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceDisks {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
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
        println!("Not implemented yet.");
        Ok(())
    }
}

/// List instances in a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceList {
    /// The project that holds the instances.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Maximum number of instances to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of instances.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        let instances = if self.paginate {
            client
                .instances()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                )
                .await?
        } else {
            client
                .instances()
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
            ctx.io.write_json(&serde_json::json!(instances))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tSTATE\tLAST UPDATED")?;
        for instance in instances {
            let last_updated = chrono::Utc::now()
                - instance
                    .time_run_state_updated
                    .unwrap_or_else(|| instance.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{}\t{}",
                &instance.name,
                &instance.description,
                &instance.run_state,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}

/// Start an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStart {
    /// The instance to start. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStart {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Start the instance.
        // TODO: Do we want a progress bar here?
        client
            .instances()
            .start(&self.instance, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Started instance {} in {}",
            cs.success_icon(),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Stop an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStop {
    /// The instance to stop. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm stop without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStop {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm stop.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm stop:", self.instance))
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

        // Stop the instance.
        // TODO: Do we want a progress bar here?
        client
            .instances()
            .stop(&self.instance, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Stopped instance {} in {}",
            cs.failure_icon_with_color(ansi_term::Color::Green),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Reboot an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceReboot {
    /// The instance to reboot. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm reboot without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceReboot {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm reboot.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm reboot:", self.instance))
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

        // Reboot the instance.
        // TODO: Do we want a progress bar here?
        client
            .instances()
            .reboot(&self.instance, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Rebooted instance {} in {}",
            cs.success_icon(),
            self.instance,
            full_name
        )?;

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
pub struct CmdInstanceView {
    /// The instance to view.
    #[clap(name = "instance", required = true)]
    pub instance: String,

    /// The project that holds the instance.
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
impl crate::cmd::Command for CmdInstanceView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}/instances/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project,
                self.instance
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let instance = client
            .instances()
            .get(&self.instance, &self.organization, &self.project)
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(instance))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", instance.id)?;
        writeln!(tw, "name:\t{}", instance.name)?;
        writeln!(tw, "description:\t{}", instance.description)?;
        writeln!(tw, "state:\t{}", instance.run_state)?;
        if let Some(run_state_updated) = instance.time_run_state_updated {
            writeln!(
                tw,
                "state updated:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - run_state_updated)
            )?;
        }
        writeln!(tw, "hostname:\t{}", instance.hostname)?;
        writeln!(tw, "ncpus:\t{}", instance.ncpus)?;
        writeln!(tw, "memory:\t{}", instance.memory)?;
        if let Some(time_created) = instance.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = instance.time_modified {
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
