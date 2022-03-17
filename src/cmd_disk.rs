use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli_macros::crud_gen;

/// Create, list, edit, view, and delete disks.
///
/// Additionally, attach and detach disks to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDisk {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
#[crud_gen {
    tag = "disks",
}]
enum SubCommand {
    Attach(CmdDiskAttach),
    Create(CmdDiskCreate),
    Delete(CmdDiskDelete),
    Detach(CmdDiskDetach),
    Edit(CmdDiskEdit),
    List(CmdDiskList),
    View(CmdDiskView),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDisk {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Attach(cmd) => cmd.run(ctx).await,
            SubCommand::Create(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Detach(cmd) => cmd.run(ctx).await,
            SubCommand::Edit(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::View(cmd) => cmd.run(ctx).await,
        }
    }
}

// TODO: on attach we could do it interactively and list the instances in the project
// as a selection list.
/// Attach a disk to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskAttach {
    /// The disk to attach. Can be an ID or name.
    #[clap(name = "disk", required = true)]
    disk: String,

    /// The instance to attach the disk to. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the disk and instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskAttach {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Attach the disk.
        client
            .instances()
            .disks_attach(
                &self.instance,
                &self.organization,
                &self.project,
                &oxide_api::types::DiskIdentifier {
                    disk: self.disk.to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Attached disk {} to instance {} in project {}",
            cs.success_icon(),
            self.disk,
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Create a new disk.
///
/// To create a disk interactively, use `oxide disk create` with no arguments.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskCreate {
    /// The name of the disk to create.
    #[clap(name = "disk", default_value = "")]
    pub disk: String,

    /// The project that will hold the disk.
    #[clap(long, short, default_value = "")]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, default_value = "", env = "OXIDE_ORG")]
    pub organization: String,

    /// The description for the disk.
    #[clap(long = "description", short = 'D', default_value = "")]
    pub description: String,

    /// The snapshot for the disk.
    #[clap(long = "snapshot", default_value = "")]
    pub snapshot: String,

    // TODO: handle human-like input for sizes.
    /// The size to allocate for the disk, in bytes.
    #[clap(long, short, default_value = "0")]
    pub size: i64,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskCreate {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let mut disk_name = self.disk.to_string();
        let mut project_name = self.project.to_string();
        let mut description = self.description.to_string();
        let mut organization = self.organization.to_string();

        let mut size = self.size;
        let mut snapshot = self.snapshot.to_string();

        if disk_name.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("[disk_name] required in non-interactive mode"));
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

        // TODO: Add this back when snapshots are supported.
        /*if snapshot.is_empty() && !ctx.io.can_prompt() {
            return Err(anyhow!("--snapshot required in non-interactive mode"));
        }*/

        if size == 0 && !ctx.io.can_prompt() {
            return Err(anyhow!("--size,-s required in non-interactive mode"));
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

        // Prompt for the disk name.
        if disk_name.is_empty() {
            match dialoguer::Input::<String>::new()
                .with_prompt("Disk name:")
                .interact_text()
            {
                Ok(name) => disk_name = name,
                Err(err) => {
                    return Err(anyhow!("prompt failed: {}", err));
                }
            }

            // Propmt for a description if they didn't provide one.
            if description.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt("Disk description:")
                    .interact_text()
                {
                    Ok(desc) => description = desc,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if snapshot.is_empty() {
                // TODO: this should be a select from all the snapshots.
                match dialoguer::Input::<String>::new()
                    .with_prompt("Disk snapshot:")
                    .interact_text()
                {
                    Ok(name) => snapshot = name,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }

            if size == 0 {
                // TODO: make this a select.
                match dialoguer::Input::<i64>::new().with_prompt("Disk size:").interact_text() {
                    Ok(m) => size = m,
                    Err(err) => {
                        return Err(anyhow!("prompt failed: {}", err));
                    }
                }
            }
        }

        let full_name = format!("{}/{}", organization, project_name);

        // Create the disk.
        client
            .disks()
            .post(
                &organization,
                &project_name,
                &oxide_api::types::DiskCreate {
                    name: disk_name.to_string(),
                    description: description.to_string(),
                    snapshot_id: snapshot.to_string(),
                    size,
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Successfully created disk {} in {}",
            cs.success_icon(),
            disk_name,
            full_name
        )?;

        Ok(())
    }
}

/// Delete a disk.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskDelete {
    /// The disk to delete. Can be an ID or name.
    #[clap(name = "disk", required = true)]
    disk: String,

    /// The project to delete the disk from.
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
impl crate::cmd::Command for CmdDiskDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm deletion.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.disk))
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
            .disks()
            .delete(&self.disk, &self.organization, &self.project)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted disk {} from {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.disk,
            full_name
        )?;

        Ok(())
    }
}

/// Detach a disk from an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskDetach {
    /// The disk to detach. Can be an ID or name.
    #[clap(name = "disk", required = true)]
    disk: String,

    /// The instance to detach the disk from. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the disk and instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskDetach {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Detach the disk.
        client
            .instances()
            .disks_detach(
                &self.instance,
                &self.organization,
                &self.project,
                &oxide_api::types::DiskIdentifier {
                    disk: self.disk.to_string(),
                },
            )
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Detached disk {} from instance {} in project {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            self.disk,
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Edit disk settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet in omicron.");
        Ok(())
    }
}

/// List disks owned by a project.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskList {
    /// The project that holds the disks.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Maximum number of disks to list.
    #[clap(long, short, default_value = "30")]
    pub limit: u32,

    /// Make additional HTTP requests to fetch all pages of disks.
    #[clap(long)]
    pub paginate: bool,

    /// Output JSON.
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskList {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.limit < 1 {
            return Err(anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;

        let disks = if self.paginate {
            client
                .disks()
                .get_all(
                    oxide_api::types::NameSortModeAscending::NameAscending,
                    &self.organization,
                    &self.project,
                )
                .await?
        } else {
            client
                .disks()
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
            ctx.io.write_json(&serde_json::json!(disks))?;
            return Ok(());
        }

        let cs = ctx.io.color_scheme();

        // TODO: add more columns, maybe make customizable.
        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "NAME\tDESCRTIPTION\tSTATE\tDEVICE PATH\tUPDATED")?;
        for disk in disks {
            let last_updated = chrono::Utc::now() - disk.time_modified.unwrap_or_else(|| disk.time_created.unwrap());
            writeln!(
                tw,
                "{}\t{}\t{}\t{}\t{}",
                &disk.name,
                &disk.description,
                &disk.state,
                &disk.device_path,
                cs.gray(&chrono_humanize::HumanTime::from(last_updated).to_string())
            )?;
        }
        tw.flush()?;

        let table = String::from_utf8(tw.into_inner()?)?;
        writeln!(ctx.io.out, "{}", table)?;

        Ok(())
    }
}

/// View a disk.
///
/// Display the description and other information of an Oxide disk.
///
/// With '--web', open the disk in a web browser instead.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskView {
    /// The disk to view.
    #[clap(name = "disk", required = true)]
    pub disk: String,

    /// The project that holds the disk.
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
impl crate::cmd::Command for CmdDiskView {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.web {
            // TODO: make sure this is the correct URL.
            let url = format!(
                "https://{}/{}/{}/disks/{}",
                ctx.config.default_host()?,
                self.organization,
                self.project,
                self.disk
            );

            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;

        let disk = client
            .disks()
            .get(&self.disk, &self.organization, &self.project)
            .await?;

        if self.json {
            // If they specified --json, just dump the JSON.
            ctx.io.write_json(&serde_json::json!(disk))?;
            return Ok(());
        }

        let mut tw = tabwriter::TabWriter::new(vec![]);
        writeln!(tw, "id:\t{}", disk.id)?;
        writeln!(tw, "name:\t{}", disk.name)?;
        writeln!(tw, "description:\t{}", disk.description)?;
        writeln!(tw, "state:\t{}", disk.state)?;
        writeln!(tw, "size:\t{}", disk.size)?;
        writeln!(tw, "device path:\t{}", disk.device_path)?;
        writeln!(tw, "snapshot:\t{}", disk.snapshot_id)?;
        if let Some(time_created) = disk.time_created {
            writeln!(
                tw,
                "created:\t{}",
                chrono_humanize::HumanTime::from(chrono::Utc::now() - time_created)
            )?;
        }
        if let Some(time_modified) = disk.time_modified {
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

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        cmd: crate::cmd_disk::SubCommand,
        stdin: String,
        want_out: String,
        want_err: String,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cmd_disk() {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "create no name".to_string(),
                cmd: crate::cmd_disk::SubCommand::Create(crate::cmd_disk::CmdDiskCreate {
                    disk: "".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    size: Default::default(),
                    snapshot: Default::default(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "[disk_name] required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no organization".to_string(),
                cmd: crate::cmd_disk::SubCommand::Create(crate::cmd_disk::CmdDiskCreate {
                    disk: "things".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    size: Default::default(),
                    snapshot: Default::default(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--organization,-o required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no project".to_string(),
                cmd: crate::cmd_disk::SubCommand::Create(crate::cmd_disk::CmdDiskCreate {
                    disk: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "".to_string(),
                    description: "".to_string(),
                    size: Default::default(),
                    snapshot: Default::default(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--project,-p required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no description".to_string(),
                cmd: crate::cmd_disk::SubCommand::Create(crate::cmd_disk::CmdDiskCreate {
                    disk: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "".to_string(),
                    size: Default::default(),
                    snapshot: Default::default(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--description,-D required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no size".to_string(),
                cmd: crate::cmd_disk::SubCommand::Create(crate::cmd_disk::CmdDiskCreate {
                    disk: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "blah blah".to_string(),
                    size: Default::default(),
                    snapshot: Default::default(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--size,-s required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "delete no --confirm non-interactive".to_string(),
                cmd: crate::cmd_disk::SubCommand::Delete(crate::cmd_disk::CmdDiskDelete {
                    disk: "things".to_string(),
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
                cmd: crate::cmd_disk::SubCommand::List(crate::cmd_disk::CmdDiskList {
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

            let cmd_disk = crate::cmd_disk::CmdDisk { subcmd: t.cmd };
            match cmd_disk.run(&mut ctx).await {
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
