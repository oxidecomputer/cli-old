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
    Edit(CmdRouterEdit),
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
