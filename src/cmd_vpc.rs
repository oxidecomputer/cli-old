use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;

/// Create, list, edit, view, and delete VPCs.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVpc {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdVpcCreate),
    Delete(CmdVpcDelete),
    Edit(CmdVpcEdit),
    List(CmdVpcList),
    View(CmdVpcView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdVpc {
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

/// Create a new VPC.
///
/// To create a VPC interactively, use `oxide vpc create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVpcCreate {
    /// The name of the VPC to create.
    #[clap(name = "vpc", default_value = "")]
    pub vpc: String,

    /// The project that will hold the VPC.
    #[clap(long, short, default_value = "")]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, default_value = "", env = "OXIDE_ORG")]
    pub organization: String,

    /// The description for the VPC.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,

    /// The dns_name for the VPC.
    #[clap(long = "dns_name", default_value = "")]
    pub dns_name: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdVpcCreate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut vpc_name = self.vpc.to_string();
        let mut project_name = self.project.to_string();
        let mut description = self.description.to_string();
        let mut organization = self.organization.to_string();

        let mut dns_name = self.dns_name.to_string();

        if (project_name.is_empty() || organization.is_empty() || vpc_name.is_empty()) && !ctx.io.can_prompt() {
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

        // Prompt for the vpc name.
        if vpc_name.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("VPC name:")
                .interact_text()
            {
                Ok(name) => vpc_name = name,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }

            // Propmt for a description if they didn't provide one.
            if description.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("VPC description:")
                    .interact_text()
                {
                    Ok(desc) => description = desc,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if dns_name.is_empty() {
                // TODO: we should generate a dns_name as the default.
                match dialoguer::Input::<String>::new()
                    .with_prompt("DNS name:")
                    .interact_text()
                {
                    Ok(name) => dns_name = name,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }
        }

        let full_name = format!("{}/{}", organization, project_name);

        // Create the disk.
        client
            .vpcs()
            .post(
                &organization,
                &project_name,
                &oxide_api::types::VpcCreate {
                    name: vpc_name.to_string(),
                    description: description.to_string(),
                    dns_name: dns_name.to_string(),
                    ipv_6_prefix: "".to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully created VPC {} in {}",
            cs.success_icon(),
            vpc_name,
            full_name
        )?;

        Ok(())
    }
}

/// Delete a VPC.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVpcDelete {
    /// The VPC to delete. Can be an ID or name.
    #[clap(name = "vpc", required = true)]
    vpc: String,

    /// The project to delete the VPC from.
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
impl crate::cmd::Command for CmdVpcDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm deletion.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.vpc))
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
            .vpcs()
            .delete(&self.vpc, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted VPC {} from {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.vpc,
            full_name
        )?;

        Ok(())
    }
}

/// Edit VPC settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVpcEdit {
    /// The VPC to edit.
    #[clap(name = "vpc", required = true)]
    pub vpc: String,

    /// The project that holds the VPC.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// The new name for the VPC.
    #[clap(long = "name", short)]
    pub new_name: Option<String>,

    /// The new description for the VPC.
    #[clap(long = "description", short = 'D')]
    pub new_description: Option<String>,

    /// The new DNS name for the VPC.
    #[clap(long = "dns_name")]
    pub new_dns_name: Option<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdVpcEdit {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.new_name.is_none() && self.new_description.is_none() && self.new_dns_name.is_none() {
            return Err(anyhow!("nothing to edit"));
        }

        let full_name = format!("{}/{}", self.organization, self.project);
        let mut name = self.vpc.clone();

        let client = ctx.api_client("")?;

        let mut body = oxide_api::types::VpcUpdate {
            name: "".to_string(),
            description: "".to_string(),
            dns_name: "".to_string(),
        };

        if let Some(n) = &self.new_name {
            body.name = n.to_string();
            // Update the name so when we spit it back out it is correct.
            name = n.to_string();
        }

        if let Some(d) = &self.new_description {
            body.description = d.to_string();
        }

        if let Some(d) = &self.new_dns_name {
            body.dns_name = d.to_string();
        }

        client
            .vpcs()
            .put(&self.organization, &self.project, &self.vpc, &body)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully edited VPC {} in {}",
            cs.success_icon(),
            name,
            full_name
        )?;

        Ok(())
    }
}

/// List VPCs owned by a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVpcList {
    /// The project that holds the VPCs.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Maximum number of VPCs to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of VPCs.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdVpcList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        let vpcs = if self.paginate {
            client
                .vpcs()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                )
                .await?
        } else {
            client
                .vpcs()
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
            ctx.io.write_json(&serde_json::json!(vpcs))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tDNS NAME\tSYSTEM ROUTER\tLAST UPDATED")?;
        for vpc in vpcs {
            let last_updated = chrono::Utc::now() - vpc.time_modified.unwrap_or_else(|| vpc.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{}\t{}\t{}",
                &vpc.name,
                &vpc.description,
                &vpc.dns_name,
                &vpc.system_router_id,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}

/// View a vpc.
///
/// Display the description and other information of an Oxide vpc.
///
/// With '--web', open the vpc in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVpcView {
    /// The VPC to view.
    #[clap(name = "vpc", required = true)]
    pub vpc: String,

    /// The project that holds the VPC.
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
impl crate::cmd::Command for CmdVpcView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}/vpcs/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project,
                self.vpc
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let vpc = client.vpcs().get(&self.vpc, &self.organization, &self.project).await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(vpc))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", vpc.id)?;
        writeln!(tw, "name:\t{}", vpc.name)?;
        writeln!(tw, "description:\t{}", vpc.description)?;
        writeln!(tw, "dns name:\t{}", vpc.dns_name)?;
        writeln!(tw, "system router:\t{}", vpc.system_router_id)?;
        if let Some(time_created) = vpc.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = vpc.time_modified {
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
