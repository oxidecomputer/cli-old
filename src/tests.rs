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
            test_host: std::env::var("OXIDE_TEST_HOST")
                .expect("you need to set OXIDE_TEST_HOST to where the api is running"),
            test_token: std::env::var("OXIDE_TEST_TOKEN").expect("OXIDE_TEST_TOKEN is required"),
        }
    }

    async fn teardown(self) {
        let oxide = oxide_api::Client::new(&self.test_token, format!("http://{}", &self.test_host));

        // Get all the orgs.
        let orgs = oxide
            .organizations()
            .get_all(oxide_api::types::NameOrIdSortMode::NameAscending)
            .await
            .unwrap();

        // Iterate over the orgs and delete all the projects.
        for org in orgs {
            // List all the projects.
            let projects = oxide
                .projects()
                .get_all(&org.name, oxide_api::types::NameOrIdSortMode::NameAscending)
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
            want_out: format!("oxide {}\n{}", version, crate::cmd_version::changelog_url(version)),
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
            name: "api endpoint does not exist".to_string(),
            args: vec!["oxide".to_string(), "api".to_string(), "foo/bar".to_string()],
            want_out: "".to_string(),
            want_err: "404 Not Found Not Found".to_string(),
            want_code: 1,
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
            want_out: "id | name | description | time_created | time_modified |
----+------+-------------+--------------+---------------"
                .to_string(),
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
            want_out: "✔ Created organization maze-war\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view an org".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "view".to_string(),
                "maze-war".to_string(),
            ],
            want_out: r#"description   | The Maze War game organization"#.to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "edit an org empty".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "edit".to_string(),
                "maze-war".to_string(),
            ],
            want_out: "".to_string(),
            want_err: "nothing to edit".to_string(),
            want_code: 1,
            ..Default::default()
        },
        TestItem {
            name: "view an org --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "view".to_string(),
                "maze-war".to_string(),
                "--json".to_string(),
            ],
            want_out: r#"{
  "description": "The Maze War game organization",
  "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "api create an org".to_string(),
            args: vec![
                "oxide".to_string(),
                "api".to_string(),
                "organizations".to_string(),
                "-F".to_string(),
                "name=zoo".to_string(),
                "-f".to_string(),
                "description=The zoo game organization".to_string(),
            ],
            want_out: r#"{
  "description": "The zoo game organization",
  "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "edit an org".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "edit".to_string(),
                "zoo".to_string(),
                "-D".to_string(),
                "The zoo 2 game organization".to_string(),
                "--name".to_string(),
                "zoo-2".to_string(),
            ],
            want_out: r#"✔ Edited organization zoo -> zoo-2"#.to_string(),
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
            want_out: "✔ Created organization dune\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "edit an org".to_string(),
            args: vec![
                "oxide".to_string(),
                "org".to_string(),
                "edit".to_string(),
                "dune".to_string(),
                "-D".to_string(),
                "The dune game organization that is in the desert".to_string(),
            ],
            want_out: r#"✔ Edited organization dune"#.to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list orgs".to_string(),
            args: vec!["oxide".to_string(), "org".to_string(), "list".to_string()],
            want_out: r#"dune   | The dune game organization that is in the desert"#.to_string(),
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
            want_out: r#"maze-war | The Maze War game organization"#.to_string(),
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
                "--organization".to_string(),
                "maze-war".to_string(),
            ],
            want_out: "id | name | description | organization_id | time_created | time_modified |
----+------+-------------+-----------------+--------------+---------------"
                .to_string(),
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
            want_out: "✔ Created project maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create another project".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "production".to_string(),
                "-D".to_string(),
                "The production project".to_string(),
            ],
            want_out: "✔ Created project maze-war/production".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list projects --json --paginate".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--json".to_string(),
                "--paginate".to_string(),
            ],
            want_out: r#""name": "production",
    "organization_id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view a project".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "development".to_string(),
            ],
            want_out: r#"description     | The development project"#.to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "edit project empty".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "edit".to_string(),
                "production".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
            ],
            want_out: "".to_string(),
            want_err: "nothing to edit".to_string(),
            want_code: 1,
            ..Default::default()
        },
        TestItem {
            name: "edit a project".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "edit".to_string(),
                "production".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "-D".to_string(),
                "The real deal prod env".to_string(),
                "--name".to_string(),
                "prod-for-reals".to_string(),
            ],
            want_out: r#"✔ Edited project maze-war/production -> maze-war/prod-for-reals"#.to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view a project --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "prod-for-reals".to_string(),
                "--json".to_string(),
            ],
            want_out: r#"{
  "description": "The real deal prod env",
  "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "edit a project".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "edit".to_string(),
                "prod-for-reals".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "-D".to_string(),
                "The realest of deals prod env".to_string(),
            ],
            want_out: r#"✔ Edited project maze-war/prod-for-reals"#.to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "delete a project".to_string(),
            args: vec![
                "oxide".to_string(),
                "project".to_string(),
                "delete".to_string(),
                "prod-for-reals".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "✔ Deleted project maze-war/prod-for-reals".to_string(),
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
            want_out: "id | name | description | hostname | memory | ncpus | project_id | run_state | time_created | time_modified | time_run_state_updated |
----+------+-------------+----------+--------+-------+------------+-----------+--------------+---------------+------------------------"
                .to_string(),
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
                "--ncpus".to_string(),
                "1".to_string(),
                "--memory".to_string(),
                "1024".to_string(),
                "--hostname".to_string(),
                "my-db".to_string(),
                "--description".to_string(),
                "My database".to_string(),
            ],
            want_out: "✔ Created instance my-db in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create another instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "my-app".to_string(),
                "--ncpus".to_string(),
                "1".to_string(),
                "--memory".to_string(),
                "1024".to_string(),
                "--hostname".to_string(),
                "my-app".to_string(),
                "--description".to_string(),
                "My application".to_string(),
            ],
            want_out: "✔ Created instance my-app in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view an instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "my-app".to_string(),
            ],
            want_out: r#"memory                 | 1024"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view an instance --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "my-app".to_string(),
                "--json".to_string(),
            ],
            want_out: r#"{
  "description": "My application",
  "hostname": "my-app",
  "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list instances --paginate --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--json".to_string(),
                "--paginate".to_string(),
            ],
            want_out: r#"[
  {
    "description": "My application",
    "hostname": "my-app",
    "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "stop an instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "stop".to_string(),
                "my-app".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "✘ Stopped instance my-app in maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "start an instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "start".to_string(),
                "my-app".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: "✔ Started instance my-app in maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "reboot an instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "reboot".to_string(),
                "my-app".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "✔ Rebooted instance my-app in maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "stop an instance again".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "stop".to_string(),
                "my-app".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "✘ Stopped instance my-app in maze-war/development".to_string(),
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
            want_out: "id | name | description | device_path | project_id | size | snapshot_id | state | time_created | time_modified |
----+------+-------------+-------------+------------+------+-------------+-------+--------------+---------------"
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        /*TestItem {
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
            want_out: "✔ Created disk new-disk in maze-war/development\n".to_string(),
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
            want_out: "✔ Created disk second-disk in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list disks --paginate --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "disk".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--json".to_string(),
                "--paginate".to_string(),
            ],
            want_out: r#"[
  {
    "description": "My new disk",
    "device_path": "/mnt/new-disk",
    "id": ""#
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
            want_out: "✔ Deleted disk second-disk from maze-war/development".to_string(),
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
            name: "list instance disks".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "disks".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "my-db".to_string(),
            ],
            want_out:
                "new-disk | My new disk"
                    .to_string(),
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
            want_out: r#"device_path   | /mnt/new-disk"#
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
        },*/
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
            want_out: r#"default | Default VPC | default  |"#.to_string(),
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
                "--dns".to_string(),
                "new-network".to_string(),
                "--description".to_string(),
                "My new network".to_string(),
            ],
            want_out: "✔ Created VPC new-network in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "create another vpc".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "create".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "netns".to_string(),
                "--dns".to_string(),
                "netns".to_string(),
                "--description".to_string(),
                "My netns network".to_string(),
            ],
            want_out: "✔ Created VPC netns in maze-war/development\n".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "edit vpc empty".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "edit".to_string(),
                "netns".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
            ],
            want_out: "".to_string(),
            want_err: "nothing to edit".to_string(),
            want_code: 1,
            ..Default::default()
        },
        TestItem {
            name: "edit a vpc".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "edit".to_string(),
                "netns".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "-D".to_string(),
                "The real deal netns".to_string(),
                "--name".to_string(),
                "netns2".to_string(),
            ],
            want_out: "✔ Edited VPC netns -> netns2 in maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view a vpc".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "netns2".to_string(),
            ],
            want_out: r#"description      | The real deal netns"#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "edit a vpc again".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "edit".to_string(),
                "netns2".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "-D".to_string(),
                "The realest of  deals netns".to_string(),
            ],
            want_out: "✔ Edited VPC netns2 in maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "view a vpc --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "view".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "netns2".to_string(),
                "--json".to_string(),
            ],
            want_out: r#"{
  "description": "The realest of  deals netns",
  "dns_name": "netns",
  "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "delete a vpc".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "delete".to_string(),
                "netns2".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "✔ Deleted VPC netns2 from maze-war/development".to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "list vpcs --paginate --json".to_string(),
            args: vec![
                "oxide".to_string(),
                "vpc".to_string(),
                "list".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--json".to_string(),
                "--paginate".to_string(),
            ],
            want_out: r#"[
  {
    "description": "Default VPC",
    "dns_name": "default",
    "id": ""#
                .to_string(),
            want_err: "".to_string(),
            want_code: 0,
            ..Default::default()
        },
        TestItem {
            name: "delete an instance".to_string(),
            args: vec![
                "oxide".to_string(),
                "instance".to_string(),
                "delete".to_string(),
                "my-app".to_string(),
                "--organization".to_string(),
                "maze-war".to_string(),
                "--project".to_string(),
                "development".to_string(),
                "--confirm".to_string(),
            ],
            want_out: "✔ Deleted instance my-app from maze-war/development".to_string(),
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
