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
            name: "api /session/me".to_string(),
            args: vec!["oxide".to_string(), "api".to_string(), "/session/me".to_string()],
            want_out: r#"{
  "id": "001de000-05e4-4000-8000-000000004007"
}"#
            .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "api session/me (no leading /)".to_string(),
            args: vec!["oxide".to_string(), "api".to_string(), "session/me".to_string()],
            want_out: r#"{
  "id": "001de000-05e4-4000-8000-000000004007"
}"#
            .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "api session/me with header".to_string(),
            args: vec![
                "oxide".to_string(),
                "api".to_string(),
                "session/me".to_string(),
                "-H".to_string(),
                "Origin: https://example.com".to_string(),
            ],
            want_out: r#"{
  "id": "001de000-05e4-4000-8000-000000004007"
}"#
            .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "api session/me with headers".to_string(),
            args: vec![
                "oxide".to_string(),
                "api".to_string(),
                "session/me".to_string(),
                "-H".to_string(),
                "Origin: https://example.com".to_string(),
                "-H".to_string(),
                "Another: thing".to_string(),
            ],
            want_out: r#"{
  "id": "001de000-05e4-4000-8000-000000004007"
}"#
            .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "api session/me with output headers".to_string(),
            args: vec![
                "oxide".to_string(),
                "api".to_string(),
                "session/me".to_string(),
                "--include".to_string(),
            ],
            want_out: r#"HTTP/1.1 200 OK
content-length:  "45"
content-type:    "application/json"
date:"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "try to paginate over a post".to_string(),
            args: vec![
                "oxide".to_string(),
                "api".to_string(),
                "organizations".to_string(),
                "--method".to_string(),
                "POST".to_string(),
                "--paginate".to_string(),
            ],
            want_out: "".to_string(),
            want_err: "the `--paginate` option is not supported for non-GET request".to_string(),
            want_code: 1,
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
            want_out: "✔ Deleted organization dune".to_string(),
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
                "instance".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: "NAME  DESCRTIPTION  STATE  UPDATED\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "my-db".to_string(),
                "--cpus".to_string(),
                "1".to_string(),
                "--memory".to_string(),
                "1024".to_string(),
                "--hostname".to_string(),
                "my-db".to_string(),
                "--description".to_string(),
                "My database".to_string(),
            ],
            want_out: "✔ Successfully created instance my-db in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list instances".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: r#"NAME   DESCRTIPTION  STATE     UPDATED
my-db  My database   starting"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list disks empty".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: "NAME  DESCRTIPTION  STATE  DEVICE PATH  UPDATED\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create disk".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "new-disk".to_string(),
                /*"--snapshot".to_string(),
                "42583766-9318-4339-A2A2-EE286F0F5B26".to_string(),*/
                "--size".to_string(),
                "1024".to_string(),
                "-D".to_string(),
                "My new disk".to_string(),
            ],
            want_out: "✔ Successfully created disk new-disk in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create another disk".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "second-disk".to_string(),
                /*"--snapshot".to_string(),
                "42583766-9318-4339-A2A2-EE286F0F5B26".to_string(),*/
                "--size".to_string(),
                "1024".to_string(),
                "-D".to_string(),
                "My second new disk".to_string(),
            ],
            want_out: "✔ Successfully created disk second-disk in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list disks".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: r#"NAME         DESCRTIPTION        STATE     DEVICE PATH       UPDATED
new-disk     My new disk         detached  /mnt/new-disk"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "delete a disk".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "delete".to_string(),
                "second-disk".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "✔ Deleted disk second-disk".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "attach a disk to an instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "attach".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "new-disk".to_string(),
                "my-db".to_string(),
            ],
            want_out: "✔ Attached disk new-disk to instance my-db in project maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view a disk".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "new-disk".to_string(),
            ],
            want_out: r#"name:         new-disk
description:  My new disk
state:        attaching
size:         1024
device path:  /mnt/new-disk
snapshot:"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view a disk --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "new-disk".to_string(),
                "--json".to_string(),
            ],
            want_out: r#"{
  "description": "My new disk",
  "device_path": "/mnt/new-disk",
  "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "detach a disk from an instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "detach".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "new-disk".to_string(),
                "my-db".to_string(),
            ],
            want_out: "✔ Detached disk new-disk from instance my-db in project maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list vpcs default".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: r#"NAME     DESCRTIPTION  DNS      SYSTEM ROUTER                         UPDATED
default  Default VPC   default"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create vpc".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "new-network".to_string(),
                "--dns-name".to_string(),
                "new-network".to_string(),
                "--description".to_string(),
                "My new network".to_string(),
            ],
            want_out: "✔ Successfully created VPC new-network in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list vpcs".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: "new-network  My new network  new-network".to_string(),
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
                assert_eq!(
                    stderr.to_string().is_empty(),
                    t.want_err.is_empty(),
                    "test {} -> stderr: {}\nwant_err: {}",
                    t.name,
                    stderr,
                    t.want_err
                );
                assert!(
                    stderr.contains(&t.want_err),
                    "test {} ->\nstderr: {}\nwant: {}\n\nstdout: {}",
                    t.name,
                    stderr,
                    t.want_err,
                    stdout,
                );
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
