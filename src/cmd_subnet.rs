use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli_macros::crud_gen;

/// Create, list, edit, view, and delete subnets.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnet {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "subnets",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdSubnetCreate),
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
pub struct CmdSubnetCreate {
    /// The name of the subnet to create.
    #[clap(name = "subnet", default_value = "")]
    pub subnet: String,

    /// The VPC that will hold the subnet.
    #[clap(long, short, default_value = "")]
    pub vpc: String,

    /// The project that holds the VPC.
    #[clap(long, short, default_value = "")]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, default_value = "", env = "OXIDE_ORG")]
    pub organization: String,

    /// The description for the subnet.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,

    /// The IPv4 block for the subnet.
    #[clap(long = "ipv4", default_value = "")]
    pub ipv4_block: String,

    /// The IPv6 block for the subnet.
    #[clap(long = "ipv6", default_value = "")]
    pub ipv6_block: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnetCreate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut subnet_name = self.subnet.to_string();
        let mut project_name = self.project.to_string();
        let mut description = self.description.to_string();
        let mut organization = self.organization.to_string();

        let mut vpc_name = self.vpc.to_string();
        let mut ipv4_block = self.ipv4_block.to_string();
        let mut ipv6_block = self.ipv6_block.to_string();

        if (project_name.is_empty() || organization.is_empty() || vpc_name.is_empty() || subnet_name.is_empty())
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

        // Prompt for the subnet name.
        if subnet_name.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Subnet name:")
                .interact_text()
            {
                Ok(name) => subnet_name = name,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }

            // Propmt for a description if they didn't provide one.
            if description.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Subnet description:")
                    .interact_text()
                {
                    Ok(desc) => description = desc,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if ipv4_block.is_empty() {
                // TODO: we should generate a ipv4_block as the default.
                match dialoguer::Input::<String>::new()
                    .with_prompt("IPv4 block:")
                    .interact_text()
                {
                    Ok(name) => ipv4_block = name,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if ipv6_block.is_empty() {
                // TODO: we should generate a ipv6_block as the default.
                match dialoguer::Input::<String>::new()
                    .with_prompt("IPv6 block:")
                    .interact_text()
                {
                    Ok(name) => ipv6_block = name,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }
        }

        let full_name = format!("{}/{}", organization, project_name);

        // Create the disk.
        client
            .subnets()
            .post(
                &organization,
                &project_name,
                &vpc_name,
                &oxide_api::types::SubnetCreate {
                    name: subnet_name.to_string(),
                    description: description.to_string(),
                    ipv4_block: ipv4_block.to_string(),
                    ipv6_block: ipv6_block.to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully created subnet {} in {} VPC {}",
            cs.success_icon(),
            subnet_name,
            full_name,
            vpc_name
        )?;

        Ok(())
    }
}

/// Edit subnet settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetEdit {
    /// The subnet to edit.
    #[clap(name = "subnet", required = true)]
    pub subnet: String,

    /// The VPC that holds the subnet.
    #[clap(long, short, required = true)]
    pub vpc: String,

    /// The project that holds the VPC.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// The new name for the subnet.
    #[clap(long = "name", short)]
    pub new_name: Option<String>,

    /// The new description for the subnet.
    #[clap(long = "description", short = 'D')]
    pub new_description: Option<String>,

    /// The new IPv4 block for the subnet.
    #[clap(long = "ipv4")]
    pub new_ipv4_block: Option<String>,

    /// The new IPv6 block for the subnet.
    #[clap(long = "ipv6")]
    pub new_ipv6_block: Option<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnetEdit {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.new_name.is_none()
            && self.new_description.is_none()
            && self.new_ipv4_block.is_none()
            && self.new_ipv6_block.is_none()
        {
            return Err(anyhow!("nothing to edit"));
        }

        let full_name = format!("{}/{}", self.organization, self.project);

        let mut name = self.subnet.clone();

        let client = ctx.api_client("")?;

        let mut body = oxide_api::types::SubnetUpdate {
            name: "".to_string(),
            description: "".to_string(),
            ipv4_block: "".to_string(),
            ipv6_block: "".to_string(),
        };

        if let Some(n) = &self.new_name {
            body.name = n.to_string();
            // Update the name so when we spit it back out it is correct.
            name = n.to_string();
        }

        if let Some(d) = &self.new_description {
            body.description = d.to_string();
        }

        if let Some(d) = &self.new_ipv4_block {
            body.ipv4_block = d.to_string();
        }

        if let Some(d) = &self.new_ipv6_block {
            body.ipv6_block = d.to_string();
        }

        client
            .subnets()
            .put(&self.organization, &self.project, &self.subnet, &self.vpc, &body)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully edited subnet {} in {} VPC {}",
            cs.success_icon(),
            name,
            full_name,
            self.vpc,
        )?;

        Ok(())
    }
}

/// List subnets owned by a VPC.
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
