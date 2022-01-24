use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;

/// Create, list, edit, view, and delete subnets.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnet {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdSubnetCreate),
    Delete(CmdSubnetDelete),
    Edit(CmdSubnetEdit),
    List(CmdSubnetList),
    View(CmdSubnetView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnet {
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

/// Create a new subnet.
///
/// To create a subnet interactively, use `oxide subnet create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetCreate {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnetCreate {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// Delete a subnet.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetDelete {
    /// The subnet to delete. Can be an ID or name.
    #[clap(name = "subnet", required = true)]
    subnet: String,

    /// The VPC that holds the subnet.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project to delete the subnet from.
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
impl crate::cmd::Command for CmdSubnetDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project,);

        // Confirm deletion.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.subnet))
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
            .subnets()
            .delete(&self.subnet, &self.organization, &self.project, &self.vpc)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted subnet {} from {} in VPC {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.subnet,
            full_name,
            self.vpc,
        )?;

        Ok(())
    }
}

/// Edit subnet settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnetEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet.");
        Ok(())
    }
}

/// List subnets owned by a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetList {
    /// The VPC that holds the subnets.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the VPC.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Maximum number of subnets to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of subnets.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnetList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        let subnets = if self.paginate {
            client
                .subnets()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                    &self.vpc,
                )
                .await?
        } else {
            client
                .subnets()
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
            ctx.io.write_json(&serde_json::json!(subnets))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tIPv4 BLOCK\tIPv6 BLOCK\tVPC\tLAST UPDATED")?;
        for subnet in subnets {
            let last_updated =
                chrono::Utc::now() - subnet.time_modified.unwrap_or_else(|| subnet.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{}\t{}\t{}\t{}",
                &subnet.name,
                &subnet.description,
                &subnet.ipv4_block,
                &subnet.ipv6_block,
                &subnet.vpc_id,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}

/// View a subnet.
///
/// Display the description and other information of an Oxide subnet.
///
/// With '--web', open the subnet in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetView {
    /// The subnet to view.
    #[clap(name = "subnet", required = true)]
    pub subnet: String,

    /// The VPC that holds the subnet.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the subnet.
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
impl crate::cmd::Command for CmdSubnetView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}/subnets/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project,
                self.subnet
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let subnet = client
            .subnets()
            .get(&self.organization, &self.project, &self.subnet, &self.vpc)
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(subnet))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", subnet.id)?;
        writeln!(tw, "name:\t{}", subnet.name)?;
        writeln!(tw, "description:\t{}", subnet.description)?;
        writeln!(tw, "ipv4 block:\t{}", subnet.ipv4_block)?;
        writeln!(tw, "ipv6 block:\t{}", subnet.ipv6_block)?;
        writeln!(tw, "vpc:\t{}", subnet.vpc_id)?;
        if let Some(time_created) = subnet.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = subnet.time_modified {
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
