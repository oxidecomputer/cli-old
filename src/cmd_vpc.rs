use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli_macros::crud_gen;

/// Create, list, edit, view, and delete VPCs.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdVpc {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "vpcs",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Create(CmdVpcCreate),
    Edit(CmdVpcEdit),
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
    #[clap(long = "dns-name", default_value = "")]
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

        if vpc_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("[vpc_name] required in non-interactive mode"));
        }

        if organization.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--organization,-o required in non-interactive mode"));
        }

        if project_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--project,-p required in non-interactive mode"));
        }

        if description.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--description,-D required in non-interactive mode"));
        }

        if dns_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--dns-name required in non-interactive mode"));
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
                .get_all(&organization, oxide_api::types::NameOrIdSortMode::NameAscending)
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

        // Create the VPC.
        client
            .vpcs()
            .post(
                &organization,
                &project_name,
                &oxide_api::types::VpcCreate {
                    name: vpc_name.to_string(),
                    description: description.to_string(),
                    dns_name: dns_name.to_string(),
                    ipv6_prefix: "".to_string(),
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
        if let Some(n) = &self.new_name {
            writeln!(
                ctx.io.out,
                "{} Successfully edited VPC {} -> {} in {}",
                cs.success_icon(),
                self.vpc,
                n,
                full_name
            )?;
        } else {
            writeln!(
                ctx.io.out,
                "{} Successfully edited VPC {} in {}",
                cs.success_icon(),
                name,
                full_name
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        cmd: crate::cmd_vpc::SubCommand,
        stdin: String,
        want_out: String,
        want_err: String,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cmd_vpc() {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "create no name".to_string(),
                cmd: crate::cmd_vpc::SubCommand::Create(crate::cmd_vpc::CmdVpcCreate {
                    vpc: "".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    dns_name: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "[vpc_name] required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no organization".to_string(),
                cmd: crate::cmd_vpc::SubCommand::Create(crate::cmd_vpc::CmdVpcCreate {
                    vpc: "things".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    dns_name: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--organization,-o required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no project".to_string(),
                cmd: crate::cmd_vpc::SubCommand::Create(crate::cmd_vpc::CmdVpcCreate {
                    vpc: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    dns_name: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--project,-p required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no description".to_string(),
                cmd: crate::cmd_vpc::SubCommand::Create(crate::cmd_vpc::CmdVpcCreate {
                    vpc: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "".to_string(),
                    dns_name: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--description,-D required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no dns_name".to_string(),
                cmd: crate::cmd_vpc::SubCommand::Create(crate::cmd_vpc::CmdVpcCreate {
                    vpc: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "blah blah".to_string(),
                    dns_name: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--dns-name required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "delete no --confirm non-interactive".to_string(),
                cmd: crate::cmd_vpc::SubCommand::Delete(crate::cmd_vpc::CmdVpcDelete {
                    vpc: "things".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    confirm: false,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--confirm required when not running interactively".to_string(),
            },
            TestItem {
                name: "list zero limit".to_string(),
                cmd: crate::cmd_vpc::SubCommand::List(crate::cmd_vpc::CmdVpcList {
                    sort_by: Default::default(),
                    limit: 0,
                    organization: "".to_string(),
                    project: "".to_string(),
                    paginate: false,
                    json: false,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--limit must be greater than 0".to_string(),
            },
        ];

        let mut config = crate::config::new_blank_config().unwrap();
        let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

        for t in tests {
            let (mut io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
            if !t.stdin.is_empty() {
                io.stdin = Box::new(std::io::Cursor::new(t.stdin));
            }
            // We need to also turn off the fancy terminal colors.
            // This ensures it also works in GitHub actions/any CI.
            io.set_color_enabled(false);
            io.set_never_prompt(true);
            let mut ctx = crate::context::Context {
                config: &mut c,
                io,
                debug: false,
            };

            let cmd_vpc = crate::cmd_vpc::CmdVpc { subcmd: t.cmd };
            match cmd_vpc.run(&mut ctx).await {
                Ok(()) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert!(stderr.is_empty(), "test {}: {}", t.name, stderr);
                    if !stdout.contains(&t.want_out) {
                        assert_eq!(stdout, t.want_out, "test {}: stdout mismatch", t.name);
                    }
                }
                Err(err) => {
                    let stdout = std::fs::read_to_string(stdout_path).unwrap();
                    let stderr = std::fs::read_to_string(stderr_path).unwrap();
                    assert_eq!(stdout, t.want_out, "test {}", t.name);
                    if !err.to_string().contains(&t.want_err) {
                        assert_eq!(err.to_string(), t.want_err, "test {}: err mismatch", t.name);
                    }
                    assert!(stderr.is_empty(), "test {}: {}", t.name, stderr);
                }
            }
        }
    }
}
