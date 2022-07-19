use std::io::Write;

use anyhow::Result;
use clap::Parser;
use cli_macro::crud_gen;

/// Create, list, edit, view, and delete organizations.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdOrganization {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "organizations",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdOrganization {
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

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use test_context::{test_context, AsyncTestContext};

    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        cmd: crate::cmd_org::SubCommand,
        stdin: String,
        want_out: String,
        want_err: String,
    }

    struct TContext {
        orig_oxide_host: Result<String, std::env::VarError>,
        orig_oxide_token: Result<String, std::env::VarError>,
    }

    #[async_trait::async_trait]
    impl AsyncTestContext for TContext {
        async fn setup() -> TContext {
            let orig = TContext {
                orig_oxide_host: std::env::var("OXIDE_HOST"),
                orig_oxide_token: std::env::var("OXIDE_TOKEN"),
            };

            // Set our test values.
            let test_host =
                std::env::var("OXIDE_TEST_HOST").expect("you need to set OXIDE_TEST_HOST to where the api is running");

            let test_token = std::env::var("OXIDE_TEST_TOKEN").expect("OXIDE_TEST_TOKEN is required");
            std::env::set_var("OXIDE_HOST", test_host);
            std::env::set_var("OXIDE_TOKEN", test_token);

            orig
        }

        async fn teardown(self) {
            // Put the original env var back.
            if let Ok(ref val) = self.orig_oxide_host {
                std::env::set_var("OXIDE_HOST", val);
            } else {
                std::env::remove_var("OXIDE_HOST");
            }

            if let Ok(ref val) = self.orig_oxide_token {
                std::env::set_var("OXIDE_TOKEN", val);
            } else {
                std::env::remove_var("OXIDE_TOKEN");
            }
        }
    }

    #[test_context(TContext)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[serial_test::serial]
    async fn test_cmd_org(_ctx: &mut TContext) {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "create no name".to_string(),
                cmd: crate::cmd_org::SubCommand::Create(crate::cmd_org::CmdOrganizationCreate {
                    organization: "".to_string(),
                    description: "hi hi".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "[organization] required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no description".to_string(),
                cmd: crate::cmd_org::SubCommand::Create(crate::cmd_org::CmdOrganizationCreate {
                    organization: "".to_string(),
                    description: "".to_string(),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "-D|--description required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "delete no --confirm non-interactive".to_string(),
                cmd: crate::cmd_org::SubCommand::Delete(crate::cmd_org::CmdOrganizationDelete {
                    organization: "things".to_string(),
                    confirm: false,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--confirm required when not running interactively".to_string(),
            },
            TestItem {
                name: "list zero limit".to_string(),
                cmd: crate::cmd_org::SubCommand::List(crate::cmd_org::CmdOrganizationList {
                    sort_by: Default::default(),
                    limit: 0,
                    paginate: false,
                    format: None,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "--limit must be greater than 0".to_string(),
            },
            TestItem {
                name: "list --json --paginate".to_string(),
                cmd: crate::cmd_org::SubCommand::List(crate::cmd_org::CmdOrganizationList {
                    sort_by: Default::default(),
                    limit: 30,
                    paginate: true,
                    format: Some(crate::types::FormatOutput::Json),
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "".to_string(),
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

            let cmd_org = crate::cmd_org::CmdOrganization { subcmd: t.cmd };
            match cmd_org.run(&mut ctx).await {
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
