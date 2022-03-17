use std::io::Write;

use anyhow::{anyhow, Result};
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
pub struct CmdProjectCreate {
    /// The name of the project to create.
    #[clap(name = "project", default_value = "")]
    pub project: String,

    // TODO: This should default to the user's org since they shouldn't be
    // able to create projects in other orgs.
    /// The organization that will hold the project.
    #[clap(long, short, env = "OXIDE_ORG", default_value = "")]
    pub organization: String,

    /// The description for the project.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,
}

// TODO: in interactive create it should default to the user's org.
#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectCreate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut project_name = self.project.to_string();
        let mut description = self.description.to_string();
        let mut organization = self.organization.to_string();

        if project_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("[project_name] required in non-interactive mode"));
        }

        if organization.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--organization,-o required in non-interactive mode"));
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

        if project_name.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Project name:")
                .interact_text()
            {
                Ok(name) => project_name = name,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }

            // Propmt for a description if they didn't provide one.
            if self.description.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Project description:")
                    .interact_text()
                {
                    Ok(desc) => description = desc,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }
        }

        let full_name = format!("{}/{}", organization, project_name);

        let client = ctx.api_client("")?;

        // Create the project.
        client
            .projects()
            .post(
                &organization,
                &oxide_api::types::RouterCreate {
                    name: project_name.to_string(),
                    description: description.to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully created project {}",
            cs.success_icon(),
            full_name
        )?;

        Ok(())
    }
}

/// Delete a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectDelete {
    /// The project to delete. Can be an ID or name.
    #[clap(name = "project", required = true)]
    pub project: String,

    /// The organization to delete the project from.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm deletion without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm deletion.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", full_name))
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
        client.projects().delete(&self.organization, &self.project).await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted project {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            full_name
        )?;

        Ok(())
    }
}

/// Edit project settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectEdit {
    /// The project to edit.
    #[clap(name = "project", required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// The new name for the project.
    #[clap(long = "name", short)]
    pub new_name: Option<String>,

    /// The new description for the project.
    #[clap(long = "description", short = 'D')]
    pub new_description: Option<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectEdit {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.new_name.is_none() && self.new_description.is_none() {
            return Err(anyhow!("nothing to edit"));
        }

        let mut full_name = format!("{}/{}", self.organization, self.project);

        let client = ctx.api_client("")?;

        let mut body = oxide_api::types::ProjectUpdate {
            name: "".to_string(),
            description: "".to_string(),
        };

        if let Some(n) = &self.new_name {
            body.name = n.to_string();
            // Update the full name, so when we print it out in the end, it's correct.
            full_name = format!("{}/{}", self.organization, n);
        }

        if let Some(d) = &self.new_description {
            body.description = d.to_string();
        }

        client.projects().put(&self.organization, &self.project, &body).await?;

        let cs = ctx.io.color_scheme();
        if self.new_name.is_some() {
            writeln!(
                ctx.io.out,
                "{} Successfully edited project {}/{} -> {}",
                cs.success_icon(),
                self.organization,
                self.project,
                full_name
            )?;
        } else {
            writeln!(
                ctx.io.out,
                "{} Successfully edited project {}",
                cs.success_icon(),
                full_name
            )?;
        }

        Ok(())
    }
}

/// List projects owned by user or organization.
///
/// If no organization is specified, the user's projects are listed.
///
/// In `--paginate` mode, all pages of results will sequentially be requested until
/// there are no more pages of results.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectList {
    /// The organization to list projects for.
    #[clap(name = "organization", required = false)]
    pub organization: String,

    /// Maximum number of projects to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of projects.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        // TODO: Make this work for a user's projects.
        let projects = if self.paginate {
            client
                .projects()
                .get_all(oxide_api::types::NameSortMode::NameAscending, &self.organization)
                .await?
        } else {
            client
                .projects()
                .get_page(
                    self.limit,
                    "",
                    oxide_api::types::NameSortMode::NameAscending,
                    &self.organization,
                )
                .await?
        };

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(projects))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tUPDATED")?;
        for project in projects {
            let full_name = format!("{}/{}", self.organization, project.name);

            let last_updated =
                chrono::Utc::now() - project.time_modified.unwrap_or_else(|| project.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{}",
                cs.bold(&full_name),
                &project.description,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

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
pub struct CmdProjectView {
    /// The project to view.
    #[clap(name = "project", required = true)]
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
impl crate::cmd::Command for CmdProjectView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let project = client.projects().get(&self.organization, &self.project).await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(project))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", project.id)?;
        writeln!(tw, "name:\t{}", project.name)?;
        writeln!(tw, "description:\t{}", project.description)?;
        writeln!(tw, "organization:\t{}", project.organization_id)?;
        if let Some(time_created) = project.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = project.time_modified {
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
