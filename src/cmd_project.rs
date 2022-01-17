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
pub struct CmdProjectDelete {
    /// The name of the project to delete.
    #[clap(name = "project", required = true)]
    pub project: String,

    /// The name of the organization to delete the project from.
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
        writeln!(ctx.io.out, "{} Deleted project {}", cs.success_icon(), full_name)?;

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
pub struct CmdProjectView {
    /// The name of the project to view.
    #[clap(name = "project", required = true)]
    pub project: String,

    /// The name of the organization to view the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Open a project in the browser.
    #[clap(short, long)]
    pub web: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            let url = format!(
                "https://{}/{}/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project
            );

            ctx.browser("", &url)?;
        }

        Ok(())
    }
}
