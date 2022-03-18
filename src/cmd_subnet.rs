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
    Edit(CmdSubnetEdit),
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
