use pretty_assertions::assert_eq;
use test_context::{test_context, AsyncTestContext};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TestItem {
    name: String,
    args: Vec<String>,
    stdin: Option<String>,
    want_out: String,
    want_err: String,
    want_code: i32,
}

struct MainContext {
    test_host: String,
    test_token: String,
}

#[async_trait::async_trait]
impl AsyncTestContext for MainContext {
    async fn setup() -> Self {
        Self {
            test_host: std::env::var("OXIDE_TEST_HOST").unwrap_or_default().to_string(),
            test_token: std::env::var("OXIDE_TEST_TOKEN").unwrap_or_default().to_string(),
        }
    }

    async fn teardown(self) {
        let oxide = oxide_api::Client::new(&self.test_token, format!("http://{}", &self.test_host));

        // Get all the orgs.
        let orgs = oxide
            .organizations()
            .get_all(oxide_api::types::NameSortMode::IdAscending)
            .await
            .unwrap();

        // Iterate over the orgs and delete all the projects.
        for org in orgs {
            // List all the projects.
            let projects = oxide
                .projects()
                .get_all(oxide_api::types::NameSortMode::IdAscending, &org.name)
                .await
                .unwrap_or_default();

            for project in projects {
                // Delete the project.
                match oxide.projects().delete(&org.name, &project.name).await {
                    Ok(_) => (),
                    Err(e) => {
                        if e.to_string().contains("404") {
                            continue;
                        }

                        panic!("Failed to delete project {}: {}", project.name, e);
                    }
                };
            }

            // Then delete the org.
            match oxide.organizations().delete(&org.name).await {
                Ok(_) => (),
                Err(e) => {
                    if e.to_string().contains("404") {
                        continue;
                    }

                    panic!("Failed to delete org {}: {}", org.name, e);
                }
            };
        }
    }
}

#[test_context(MainContext)]
#[tokio::test]
async fn test_main(ctx: &mut MainContext) {
    let version = clap::crate_version!();

    let tests: Vec<TestItem> = vec![
        TestItem {
            name: "existing command".to_string(),
            args: vec!["oxide".to_string(), "completion".to_string()],
            want_out: "complete -F _oxide -o bashdefault -o default oxide\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "existing command with args".to_string(),
            args: vec![
                "oxide".to_string(),
                "completion".to_string(),
                "-s".to_string(),
                "zsh".to_string(),
            ],
            want_out: "_oxide \"$@\"\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "add an alias".to_string(),
            args: vec![
                "oxide".to_string(),
                "alias".to_string(),
                "set".to_string(),
                "foo".to_string(),
                "completion -s zsh".to_string(),
            ],
            want_out: "- Adding alias for foo: completion -s zsh\n✔ Added alias.".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "add a shell alias".to_string(),
            args: vec![
                "oxide".to_string(),
                "alias".to_string(),
                "set".to_string(),
                "-s".to_string(),
                "bar".to_string(),
                "which bash".to_string(),
            ],
            want_out: "- Adding alias for bar: !which bash\n✔ Added alias.".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list our aliases".to_string(),
            args: vec!["oxide".to_string(), "alias".to_string(), "list".to_string()],
            want_out: "\"completion -s zsh\"".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "call alias".to_string(),
            args: vec!["oxide".to_string(), "foo".to_string()],
            want_out: "_oxide \"$@\"\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "call alias with different binary name".to_string(),
            args: vec!["/bin/thing/oxide".to_string(), "foo".to_string()],
            want_out: "_oxide \"$@\"\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "call shell alias".to_string(),
            args: vec!["oxide".to_string(), "bar".to_string()],
            want_out: "/bash".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "version".to_string(),
            args: vec!["oxide".to_string(), "version".to_string()],
            want_out: format!("oxide {}\n{}", version, crate::cmd_version::changelog_url(&version)),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "login".to_string(),
            args: vec![
                "oxide".to_string(),
                "auth".to_string(),
                "login".to_string(),
                "--host".to_string(),
                ctx.test_host.clone(),
                "--with-token".to_string(),
            ],
            stdin: Some(ctx.test_token.clone()),
            want_out: "✔ Logged in as ".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list orgs empty".to_string(),
            args: vec!["oxide".to_string(), "org".to_string(), "list".to_string()],
            want_out: "NAME  DESCRTIPTION  UPDATED\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create an org".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "create".to_string(),
                "maze-war".to_string(),
                "-D".to_string(),
                "The Maze War game organization".to_string(),
            ],
            want_out: "✔ Successfully created organization maze-war\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create another org".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "create".to_string(),
                "dune".to_string(),
                "-D".to_string(),
                "A sandy desert game".to_string(),
            ],
            want_out: "✔ Successfully created organization dune\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list orgs".to_string(),
            args: vec!["oxide".to_string(), "org".to_string(), "list".to_string()],
            want_out: r#"NAME      DESCRTIPTION                    UPDATED
dune      A sandy desert game             "#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "delete an org".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "delete".to_string(),
                "dune".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "Deleted organization dune".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list orgs after delete".to_string(),
            args: vec!["oxide".to_string(), "org".to_string(), "list".to_string()],
            want_out: r#"NAME      DESCRTIPTION                    UPDATED
maze-war  The Maze War game organization"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list projects empty".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "list".to_string(),
                "maze-war".to_string(),
            ],
            want_out: "NAME  DESCRTIPTION  UPDATED\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create a project".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "development".to_string(),
                "-D".to_string(),
                "The development project".to_string(),
            ],
            want_out: "✔ Successfully created project maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list projects".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "list".to_string(),
                "maze-war".to_string(),
            ],
            want_out: r#"NAME                  DESCRTIPTION             UPDATED
maze-war/development  The development project"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list instances empty".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "list".to_string(),
                "maze-war".to_string(),
            ],
            want_out: "NAME  DESCRTIPTION  STATE  UPDATED\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
    ];

    let mut config = crate::config::new_blank_config().unwrap();
    let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

    for t in tests {
        let (mut io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
        io.set_stdout_tty(false);
        io.set_color_enabled(false);
        if let Some(stdin) = t.stdin {
            io.stdin = Box::new(std::io::Cursor::new(stdin));
        }
        let mut ctx = crate::context::Context {
            config: &mut c,
            io,
            debug: false,
        };

        let result = crate::do_main(t.args, &mut ctx).await;

        let stdout = std::fs::read_to_string(stdout_path).unwrap_or_default();
        let stderr = std::fs::read_to_string(stderr_path).unwrap_or_default();

        assert!(
            stdout.contains(&t.want_out),
            "test {} ->\nstdout: {}\nwant: {}\n\nstderr: {}",
            t.name,
            stdout,
            t.want_out,
            stderr,
        );

        match result {
            Ok(code) => {
                assert_eq!(code, t.want_code, "test {}", t.name);
                assert_eq!(stdout.is_empty(), t.want_out.is_empty(), "test {}", t.name);
                assert!(stderr.is_empty(), "test {}", t.name);
            }
            Err(err) => {
                assert!(!t.want_err.is_empty(), "test {}", t.name);
                assert!(
                    err.to_string().contains(&t.want_err),
                    "test {} -> err: {}\nwant_err: {}",
                    t.name,
                    err,
                    t.want_err
                );
                assert_eq!(
                    err.to_string().is_empty(),
                    t.want_err.is_empty(),
                    "test {} -> err: {}\nwant_err: {}",
                    t.name,
                    err,
                    t.want_err
                );
                assert!(stderr.is_empty(), "test {}", t.name);
            }
        }
    }
}
