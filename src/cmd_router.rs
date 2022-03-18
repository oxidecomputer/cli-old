use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli_macros::crud_gen;

/// Create, list, edit, view, and delete routers.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouter {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "routers",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdRouterCreate),
    Edit(CmdRouterEdit),
    List(CmdRouterList),
    View(CmdRouterView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouter {
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

/// Create a new router.
///
/// To create a router interactively, use `oxide router create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouterCreate {
    /// The name of the router to create.
    #[clap(name = "router", default_value = "")]
    pub router: String,

    /// The VPC that will hold the router.
    #[clap(long, short, default_value = "")]
    pub vpc: String,

    /// The project that holds the VPC.
    #[clap(long, short, default_value = "")]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, default_value = "", env = "OXIDE_ORG")]
    pub organization: String,

    /// The description for the router.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouterCreate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut router_name = self.router.to_string();
        let mut project_name = self.project.to_string();
        let mut description = self.description.to_string();
        let mut organization = self.organization.to_string();

        let mut vpc_name = self.vpc.to_string();

        if (project_name.is_empty() || organization.is_empty() || vpc_name.is_empty() || router_name.is_empty())
            && !ctx.io.can_prompt()
        {
            return Err(anyhow!("at least one argument required in non-interactive mode"));
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

        // Select the VPC from the selected project.
        if vpc_name.is_empty() {
            let mut pvpcs: Vec<String> = Vec::new();
            let vpcs = client
                .vpcs()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &organization,
                    &project_name,
                )
                .await?;
            for vpc in vpcs {
                pvpcs.push(vpc.name.to_string());
            }

            // Select the vpc from the selected project.
            match dialoguer::Select::new()
                .with_prompt("Select VPC:")
                .items(&pvpcs)
                .interact()
            {
                Ok(index) => vpc_name = pvpcs[index].to_string(),
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }
        }

        // Prompt for the router name.
        if router_name.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Router name:")
                .interact_text()
            {
                Ok(name) => router_name = name,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }

            // Propmt for a description if they didn't provide one.
            if description.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Router description:")
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

        // Create the disk.
        client
            .routers()
            .post(
                &organization,
                &project_name,
                &vpc_name,
                &oxide_api::types::RouterCreate {
                    name: router_name.to_string(),
                    description: description.to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully created router {} in {} VPC {}",
            cs.success_icon(),
            router_name,
            full_name,
            vpc_name
        )?;

        Ok(())
    }
}

/// Edit router settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouterEdit {
    /// The router to edit.
    #[clap(name = "router", required = true)]
    pub router: String,

    /// The VPC that holds the router.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the VPC.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// The new name for the router.
    #[clap(long = "name", short)]
    pub new_name: Option<String>,

    /// The new description for the router.
    #[clap(long = "description", short = 'D')]
    pub new_description: Option<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouterEdit {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.new_name.is_none() && self.new_description.is_none() {
            return Err(anyhow!("nothing to edit"));
        }

        let full_name = format!("{}/{}", self.organization, self.project);

        let mut name = self.router.clone();

        let client = ctx.api_client("")?;

        let mut body = oxide_api::types::RouterUpdate {
            name: "".to_string(),
            description: "".to_string(),
        };

        if let Some(n) = &self.new_name {
            body.name = n.to_string();
            // Update the name so when we spit it back out it is correct.
            name = n.to_string();
        }

        if let Some(d) = &self.new_description {
            body.description = d.to_string();
        }

        client
            .routers()
            .put(&self.organization, &self.project, &self.router, &self.vpc, &body)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully edited router {} in {} VPC {}",
            cs.success_icon(),
            name,
            full_name,
            self.vpc,
        )?;

        Ok(())
    }
}

/// List routers owned by a VPC.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouterList {
    /// The VPC that holds the routers.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the routers.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Maximum number of routers to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of routers.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouterList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        let routers = if self.paginate {
            client
                .routers()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                    &self.vpc,
                )
                .await?
        } else {
            client
                .routers()
                .get_page(
                    self.limit,
                    "",
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                    &self.vpc,
                )
                .await?
        };

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(routers))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tKIND\tVPC\tLAST UPDATED")?;
        for router in routers {
            let last_updated =
                chrono::Utc::now() - router.time_modified.unwrap_or_else(|| router.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{}\t{}\t{}",
                &router.name,
                &router.description,
                &router.kind,
                &router.vpc_id,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}

/// View a router.
///
/// Display the description and other information of an Oxide router.
///
/// With '--web', open the router in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouterView {
    /// The router to view.
    #[clap(name = "router", required = true)]
    pub router: String,

    /// The VPC that holds the router.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the router.
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
impl crate::cmd::Command for CmdRouterView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}/routers/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project,
                self.router
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let router = client
            .routers()
            .get(&self.router, &self.organization, &self.project, &self.vpc)
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(router))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", router.id)?;
        writeln!(tw, "name:\t{}", router.name)?;
        writeln!(tw, "description:\t{}", router.description)?;
        writeln!(tw, "kind:\t{}", router.kind)?;
        writeln!(tw, "vpc:\t{}", router.vpc_id)?;
        if let Some(time_created) = router.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = router.time_modified {
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
