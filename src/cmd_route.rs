use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;

/// Create, list, edit, view, and delete routes.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRoute {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdRouteCreate),
    Delete(CmdRouteDelete),
    Edit(CmdRouteEdit),
    List(CmdRouteList),
    View(CmdRouteView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRoute {
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

/// Create a new route.
///
/// To create a route interactively, use `oxide route create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouteCreate {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouteCreate {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// Delete a route.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouteDelete {
    /// The route to delete. Can be an ID or name.
    #[clap(name = "route", required = true)]
    route: String,

    /// The router the route belongs to.
    #[clap(long, short, required = true)]
    pub router: String,

    /// The VPC that holds the route.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project to delete the route from.
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
impl crate::cmd::Command for CmdRouteDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm deletion.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.route))
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
            .routes()
            .delete(&self.organization, &self.project, &self.route, &self.router, &self.vpc)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted route {} from {} in VPC {} and router {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.route,
            full_name,
            self.router,
            self.vpc
        )?;

        Ok(())
    }
}

/// Edit route settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouteEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouteEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// List routes owned by a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouteList {
    /// The router the routes belong to.
    #[clap(long, short, required = true)]
    pub router: String,

    /// The VPC that holds the router.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the routes.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Maximum number of routes to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of routes.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdRouteList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        let routes = if self.paginate {
            client
                .routes()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                    &self.router,
                    &self.vpc,
                )
                .await?
        } else {
            client
                .routes()
                .get_page(
                    self.limit,
                    "",
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                    &self.router,
                    &self.vpc,
                )
                .await?
        };

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(routes))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(
            tw,
            "NAME\tDESCRTIPTION\tKIND\tDESTINATION\tTARGET\tROUTER\tLAST UPDATED"
        )?;
        for route in routes {
            let last_updated = chrono::Utc::now() - route.time_modified.unwrap_or_else(|| route.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                &route.name,
                &route.description,
                &route.kind,
                &route.destination,
                &route.target,
                &route.router_id,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}

/// View a route.
///
/// Display the description and other information of an Oxide route.
///
/// With '--web', open the route in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdRouteView {
    /// The route to view.
    #[clap(name = "route", required = true)]
    pub route: String,

    /// The router the route belongs to.
    #[clap(long, short, required = true)]
    pub router: String,

    /// The VPC that holds the route.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the route.
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
impl crate::cmd::Command for CmdRouteView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}/routes/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project,
                self.route
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let route = client
            .routes()
            .get(&self.organization, &self.project, &self.route, &self.router, &self.vpc)
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(route))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", route.id)?;
        writeln!(tw, "name:\t{}", route.name)?;
        writeln!(tw, "description:\t{}", route.description)?;
        writeln!(tw, "kind:\t{}", route.kind)?;
        writeln!(tw, "destination:\t{}", route.destination)?;
        writeln!(tw, "target:\t{}", route.target)?;
        writeln!(tw, "router:\t{}", route.router_id)?;
        if let Some(time_created) = route.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = route.time_modified {
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
