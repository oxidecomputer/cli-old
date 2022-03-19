use std::io::Write;

use anyhow::Result;
use clap::Parser;
use cli_macro::crud_gen;

/// Create, list, edit, view, and delete disks.
///
/// Additionally, attach and detach disks to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDisk {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "disks",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Attach(CmdDiskAttach),
    Detach(CmdDiskDetach),
    Edit(CmdDiskEdit),
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
                    description: "hi hi".to_string(),
                    size: Default::default(),
                    snapshot: Default::default(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "[disk] required in non-interactive mode".to_string(),
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
                    sort_by: Default::default(),
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
