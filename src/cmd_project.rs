use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli_macro::crud_gen;

/// Create, list, edit, view, and delete projects.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProject {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "projects",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Edit(CmdProjectEdit),
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

/// Edit project settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdProjectEdit {
    /// The project to edit.
    #[clap(name = "project", required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// The new name for the project.
    #[clap(long = "name", short)]
    pub new_name: Option<String>,

    /// The new description for the project.
    #[clap(long = "description", short = 'D')]
    pub new_description: Option<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdProjectEdit {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.new_name.is_none() && self.new_description.is_none() {
            return Err(anyhow!("nothing to edit"));
        }

        let mut full_name = format!("{}/{}", self.organization, self.project);

        let client = ctx.api_client("")?;

        let mut body = oxide_api::types::ProjectUpdate {
            name: "".to_string(),
            description: "".to_string(),
        };

        if let Some(n) = &self.new_name {
            body.name = n.to_string();
            // Update the full name, so when we print it out in the end, it's correct.
            full_name = format!("{}/{}", self.organization, n);
        }

        if let Some(d) = &self.new_description {
            body.description = d.to_string();
        }

        client.projects().put(&self.organization, &self.project, &body).await?;

        let cs = ctx.io.color_scheme();
        if self.new_name.is_some() {
            writeln!(
                ctx.io.out,
                "{} Successfully edited project {}/{} -> {}",
                cs.success_icon(),
                self.organization,
                self.project,
                full_name
            )?;
        } else {
            writeln!(
                ctx.io.out,
                "{} Successfully edited project {}",
                cs.success_icon(),
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
        cmd: crate::cmd_project::SubCommand,
        stdin: String,
        want_out: String,
        want_err: String,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cmd_project() {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "create no name".to_string(),
                cmd: crate::cmd_project::SubCommand::Create(crate::cmd_project::CmdProjectCreate {
                    project: "".to_string(),
                    organization: "".to_string(),
                    description: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "[project_name] required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no organization".to_string(),
                cmd: crate::cmd_project::SubCommand::Create(crate::cmd_project::CmdProjectCreate {
                    project: "things".to_string(),
                    organization: "".to_string(),
                    description: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--organization,-o required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no description".to_string(),
                cmd: crate::cmd_project::SubCommand::Create(crate::cmd_project::CmdProjectCreate {
                    project: "things".to_string(),
                    organization: "foo".to_string(),
                    description: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--description,-D required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "delete no --confirm non-interactive".to_string(),
                cmd: crate::cmd_project::SubCommand::Delete(crate::cmd_project::CmdProjectDelete {
                    project: "things".to_string(),
                    organization: "".to_string(),
                    confirm: false,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--confirm required when not running interactively".to_string(),
            },
            TestItem {
                name: "list zero limit".to_string(),
                cmd: crate::cmd_project::SubCommand::List(crate::cmd_project::CmdProjectList {
                    sort_by: Default::default(),
                    limit: 0,
                    organization: "".to_string(),
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

            let cmd_project = crate::cmd_project::CmdProject { subcmd: t.cmd };
            match cmd_project.run(&mut ctx).await {
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
