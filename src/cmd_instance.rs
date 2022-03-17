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
pub struct CmdInstanceCreate {
    /// The name of the instance to create.
    #[clap(name = "instance", default_value = "")]
    pub instance: String,

    /// The project that will hold the instance.
    #[clap(long, short, default_value = "")]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, default_value = "", env = "OXIDE_ORG")]
    pub organization: String,

    /// The description for the instance.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,

    /// The hostname for the instance.
    #[clap(long = "hostname", short = 'H', default_value = "")]
    pub hostname: String,

    // TODO: handle human-like input for sizes.
    /// The memory to allocate for the instance, in bytes.
    #[clap(long, short, default_value = "0")]
    pub memory: i64,

    /// The number of CPUs to allocate for the instance.
    #[clap(long, short, default_value = "0")]
    pub cpus: i64,
}

// TODO: in interactive create it should list the projects from the user's org as a select.
#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceCreate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut instance_name = self.instance.to_string();
        let mut project_name = self.project.to_string();
        let mut description = self.description.to_string();
        let mut organization = self.organization.to_string();

        let mut ncpus = self.cpus;
        let mut memory = self.memory;
        let mut hostname = self.hostname.to_string();

        if project_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--project,-p required in non-interactive mode"));
        }

        if organization.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--organization,-o required in non-interactive mode"));
        }

        if instance_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("[instance_name] required in non-interactive mode"));
        }

        if ncpus == 0 && !ctx.io.can_prompt() {
            return Err(anyhow!("--cpus,-c required in non-interactive mode"));
        }

        if memory == 0 && !ctx.io.can_prompt() {
            return Err(anyhow!("--memory,m required in non-interactive mode"));
        }

        if hostname.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--hostname,-H required in non-interactive mode"));
        }

        if description.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--description,-D required in non-interactive mode"));
        }

        // If they didn't specify an organization, prompt for it.
        if organization.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Project organization:")
                .interact_text()
            {
                Ok(org) => organization = org,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }
        }

        let client = ctx.api_client("")?;

        if project_name.is_empty() {
            let mut org_projects: Vec<String> = Vec::new();
            let projects = client
                .projects()
                .get_all(oxide_api::types::NameSortMode::NameAscending, &organization)
                .await?;
            for project in projects {
                org_projects.push(project.name.to_string());
            }

            // Select the project from the selected organization.
            match dialoguer::Select::new()
                .with_prompt("Select project:")
                .items(&org_projects)
                .interact()
            {
                Ok(index) => project_name = org_projects[index].to_string(),
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }
        }

        // Prompt for the instance name.
        if instance_name.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Instance name:")
                .interact_text()
            {
                Ok(name) => instance_name = name,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }

            // Propmt for a description if they didn't provide one.
            if description.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Instance description:")
                    .interact_text()
                {
                    Ok(desc) => description = desc,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if hostname.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Instance hostname:")
                    .interact_text()
                {
                    Ok(name) => hostname = name,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if memory == 0 {
                // TODO: make this a select.
                match dialoguer::Input::<i64>::new()
                    .with_prompt("Instance memory:")
                    .interact_text()
                {
                    Ok(m) => memory = m,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if ncpus == 0 {
                // TODO: make this a select.
                match dialoguer::Input::<i64>::new()
                    .with_prompt("Instance CPUs:")
                    .interact_text()
                {
                    Ok(m) => ncpus = m,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }
        }

        let full_name = format!("{}/{}", organization, project_name);

        // Create the disk.
        client
            .instances()
            .post(
                &organization,
                &project_name,
                &oxide_api::types::InstanceCreate {
                    name: instance_name.to_string(),
                    description: description.to_string(),
                    hostname: hostname.to_string(),
                    memory,
                    ncpus,
                    network_interfaces: Default::default(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully created instance {} in {}",
            cs.success_icon(),
            instance_name,
            full_name
        )?;

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
pub struct CmdInstanceDisks {
    /// The instance to view the disks for.
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
impl crate::cmd::Command for CmdInstanceDisks {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let disks = client
            .instances()
            .disks_get_all(
                oxide_api::types::NameSortModeAscending::NameAscending,
                &self.instance,
                &self.organization,
                &self.project,
            )
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(disks))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        // TODO: for state the api lib should implement display
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tSTATE\tDEVICE PATH\tUPDATED")?;
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

/// Edit instance settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet in omicron.");
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
        writeln!(tw, "NAME\tDESCRTIPTION\tSTATE\tUPDATED")?;
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
