use std::io::Write;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli_macro::crud_gen;

/// Create, list, edit, view, and delete instances.
///
/// Additionally, start, stop, and reboot instances.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstance {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[crud_gen {
    tag = "instances",
}]
#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Disks(CmdInstanceDisks),
    Edit(CmdInstanceEdit),
    Ssh(CmdInstanceSsh),
    Start(CmdInstanceStart),
    Stop(CmdInstanceStop),
    Reboot(CmdInstanceReboot),
    Serial(CmdInstanceSerial),
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstance {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Create(cmd) => cmd.run(ctx).await,
            SubCommand::Delete(cmd) => cmd.run(ctx).await,
            SubCommand::Disks(cmd) => cmd.run(ctx).await,
            SubCommand::Edit(cmd) => cmd.run(ctx).await,
            SubCommand::List(cmd) => cmd.run(ctx).await,
            SubCommand::Serial(cmd) => cmd.run(ctx).await,
            SubCommand::Ssh(cmd) => cmd.run(ctx).await,
            SubCommand::Start(cmd) => cmd.run(ctx).await,
            SubCommand::Stop(cmd) => cmd.run(ctx).await,
            SubCommand::Reboot(cmd) => cmd.run(ctx).await,
            SubCommand::View(cmd) => cmd.run(ctx).await,
        }
    }
}

/// List the disks attached to an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceDisks {
    /// The instance to view the disks for.
    #[clap(name = "instance", required = true)]
    pub instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization to view the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    #[doc = r" Output format."]
    #[clap(long, short)]
    pub format: Option<crate::types::FormatOutput>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceDisks {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let results = client
            .instances()
            .disks_get_all(
                &self.instance,
                &self.organization,
                &self.project,
                oxide_api::types::NameSortMode::NameAscending,
            )
            .await?;

        let format = ctx.format(&self.format)?;
        ctx.io.write_output_for_vec(&format, &results)?;
        Ok(())
    }
}

/// Edit instance settings.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceEdit {}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceEdit {
    async fn run(&self, _ctx: &mut crate::context::Context) -> Result<()> {
        println!("Not implemented yet in omicron.");
        Ok(())
    }
}

/// Start an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStart {
    /// The instance to start. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStart {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Start the instance.
        client
            .instances()
            .start(&self.instance, &self.organization, &self.project)
            .await?;

        // Wait for the instance to be started.
        let instance_state = InstanceDetails {
            host: "".to_string(),
            instance: self.instance.to_string(),
            organization: self.organization.to_string(),
            project: self.project.to_string(),
        };

        instance_state
            .wait_for_state(ctx, oxide_api::types::InstanceState::Running)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Started instance {} in {}",
            cs.success_icon(),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Stop an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceStop {
    /// The instance to stop. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm stop without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceStop {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm stop.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm stop:", self.instance))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == self.instance {
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

        // Stop the instance.
        client
            .instances()
            .stop(&self.instance, &self.organization, &self.project)
            .await?;

        // Wait for the instance to be stopped.
        let instance_state = InstanceDetails {
            host: "".to_string(),
            instance: self.instance.to_string(),
            organization: self.organization.to_string(),
            project: self.project.to_string(),
        };

        instance_state
            .wait_for_state(ctx, oxide_api::types::InstanceState::Stopped)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Stopped instance {} in {}",
            cs.failure_icon_with_color(ansi_term::Color::Green),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

/// Reboot an instance.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdInstanceReboot {
    /// The instance to reboot. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// Confirm reboot without prompting.
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceReboot {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow!("--confirm required when not running interactively"));
        }

        let client = ctx.api_client("")?;

        let full_name = format!("{}/{}", self.organization, self.project);

        // Confirm reboot.
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm reboot:", self.instance))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == self.instance {
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

        // Reboot the instance.
        client
            .instances()
            .reboot(&self.instance, &self.organization, &self.project)
            .await?;

        // Wait for the instance to be started.
        let instance_state = InstanceDetails {
            host: "".to_string(),
            instance: self.instance.to_string(),
            organization: self.organization.to_string(),
            project: self.project.to_string(),
        };

        instance_state
            .wait_for_state(ctx, oxide_api::types::InstanceState::Running)
            .await?;

        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Rebooted instance {} in {}",
            cs.success_icon(),
            self.instance,
            full_name
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct InstanceDetails {
    host: String,
    organization: String,
    project: String,
    instance: String,
}

impl InstanceDetails {
    async fn wait_for_state(
        &self,
        ctx: &mut crate::context::Context<'_>,
        status: oxide_api::types::InstanceState,
    ) -> Result<()> {
        // Start the progress bar.
        let handle = ctx
            .io
            .start_process_indicator_with_label(&format!(" Waiting for instance status to be `{}`", status));

        let client = ctx.api_client(&self.host)?;

        // TODO: we should probably time out here eventually with an error.
        loop {
            // Get the instance.
            let instance = client
                .instances()
                .get(&self.instance, &self.organization, &self.project)
                .await?;
            if status == instance.run_state {
                break;
            }

            // Back off a bit.
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // End the progress bar.
        if let Some(handle) = handle {
            handle.done();
        }

        Ok(())
    }
}

/// SSH into an instance.
///
/// This command is a thin wrapper around the **ssh(1)** command that takes care of
/// authentication and the translation of the instance name into an IP address.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment, trailing_var_arg = true)]
pub struct CmdInstanceSsh {
    /// The instance to ssh into. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    pub instance: String,

    /// The command and args to run on the instance. If not specified,
    /// you will get an interactive shell.
    #[clap(name = "args", multiple_values = true, required = false)]
    pub args: Vec<String>,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// The ssh user. This defaults to `$USER` on the host the command is run on.
    #[clap(long, short, required = true, env = "USER")]
    pub user: String,

    /// Additional flags to be passed to **ssh(1)**. It is recommended that flags
    /// be passed using an assignment operator and quotes.
    /// Example: `--ssh-flag "-L 80:localhost:80"`.
    #[clap(long = "ssh-flag", multiple_occurrences = true, required = false)]
    pub ssh_flags: Vec<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceSsh {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        // Generate a key to use for ssh-ing into the instance.
        // We default to ed25519 here, since its a nice thing.
        writeln!(ctx.io.out, "Generating a temporary ssh key...")?;
        /* let key = thrussh_keys::key::KeyPair::generate_ed25519().unwrap();
                let pubkey = key.clone_public_key();

                writeln!(
                    ctx.io.out,
                    "Temporary public key has fingerprint `{}`",
                    pubkey.fingerprint()
                )?;

                writeln!(
                    ctx.io.out,
                    "Temporary bytes are `ssh-ed25519 {}`",
                    pubkey.public_key_base64()
                )?;

                println!("ARGS: {:?}", self.args);

                // TODO: Add our pubkey to our Oxide user's authorized_keys.
                writeln!(ctx.io.out, "Adding temporary ssh key to your user account...")?;
        */
        // TODO: We need to get the instance IP address.
        let _instance = client
            .instances()
            .get(&self.instance, &self.organization, &self.project)
            .await?;

        // Wrap the ssh command in a shell.
        std::process::Command::new("ssh")
            //.arg(host)
            .args(&self.args)
            .stdout(std::process::Stdio::inherit())
            .stdin(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()?;

        // TODO: When we are done, we need to remove our key from our Oxide user's authorized keys.
        // This makes it act as a temporary key.
        writeln!(
            ctx.io.out,
            "Cleaning up the temporary ssh key from your user account..."
        )?;

        Ok(())
    }
}

/// Read the buffered data from an instance's serial console.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment, trailing_var_arg = true)]
pub struct CmdInstanceSerial {
    /// The instance whose serial console we wish to view. Can be an ID or name.
    #[clap(name = "instance", required = true)]
    pub instance: String,

    /// The project that holds the instance.
    #[clap(long, short, required = true)]
    pub project: String,

    /// The organization that holds the project.
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,

    /// The maximum length of bytes to retrieve.
    #[clap(long, short)]
    pub max_bytes: Option<u64>,

    /// The offset since boot (or if negative, the current end of the buffered data) from which to
    /// retrieve output. Defaults to the most recent 16 KiB of serial console output (-16384).
    #[clap(long, short)]
    pub byte_offset: Option<i64>,

    /// Whether to continuously read from the running instance's output.
    #[clap(long, short)]
    pub continuous: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdInstanceSerial {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let client = ctx.api_client("")?;

        let mut from_start = None;
        let mut most_recent = None;
        let max_bytes = self.max_bytes;

        match self.byte_offset {
            Some(x) if x >= 0 => from_start = Some(x as u64),
            Some(x) => most_recent = Some(-x as u64),
            None => most_recent = Some(16384),
        }

        let mut cont = true;
        while cont {
            let output = client
                .instances()
                .serial_get(
                    from_start,
                    &self.instance,
                    max_bytes,
                    most_recent,
                    &self.organization,
                    &self.project,
                )
                .await?;

            std::io::stdout().write_all(&output.data)?;

            cont = self.continuous;
            most_recent = None;
            from_start = Some(output.last_byte_offset);

            if cont && output.data.is_empty() {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        }

        println!("\x1b[0m");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::cmd::Command;

    pub struct TestItem {
        name: String,
        cmd: crate::cmd_instance::SubCommand,
        stdin: String,
        want_out: String,
        want_err: String,
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cmd_instance() {
        let tests: Vec<TestItem> = vec![
            TestItem {
                name: "create no name".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "hi hi".to_string(),
                    memory: 1024,
                    ncpus: 2,
                    hostname: "holla".to_string(),
                    network_interfaces: Default::default(),
                    disks: Default::default(),
                    user_data: "some data".to_string(),
                    external_ips: Vec::from(["mypool".to_string()]),
                    start: true,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "[instance] required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no organization".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "".to_string(),
                    project: "".to_string(),
                    description: "hi hi".to_string(),
                    memory: 1024,
                    ncpus: 2,
                    hostname: "holla".to_string(),
                    network_interfaces: Default::default(),
                    disks: Default::default(),
                    user_data: "some data".to_string(),
                    external_ips: Vec::from(["mypool".to_string()]),
                    start: true,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "-o|--organization required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no project".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "blah".to_string(),
                    project: "".to_string(),
                    description: "hi hi".to_string(),
                    memory: 1024,
                    ncpus: 2,
                    hostname: "holla".to_string(),
                    network_interfaces: Default::default(),
                    disks: Default::default(),
                    user_data: "some data".to_string(),
                    external_ips: Vec::from(["mypool".to_string()]),
                    start: true,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "-p|--project required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no description".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "".to_string(),
                    memory: 0,
                    ncpus: 0,
                    hostname: "".to_string(),
                    network_interfaces: Default::default(),
                    disks: Default::default(),
                    user_data: "some data".to_string(),
                    external_ips: Vec::from(["mypool".to_string()]),
                    start: true,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "D|--description required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no cpus".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "blah blah".to_string(),
                    memory: 1024,
                    ncpus: 0,
                    hostname: "sup".to_string(),
                    network_interfaces: Default::default(),
                    disks: Default::default(),
                    user_data: "some data".to_string(),
                    external_ips: Vec::from(["mypool".to_string()]),
                    start: true,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "-c|--ncpus required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "create no memory".to_string(),
                cmd: crate::cmd_instance::SubCommand::Create(crate::cmd_instance::CmdInstanceCreate {
                    instance: "things".to_string(),
                    organization: "foo".to_string(),
                    project: "bar".to_string(),
                    description: "blah blah".to_string(),
                    memory: 0,
                    ncpus: 2,
                    hostname: "sup".to_string(),
                    network_interfaces: Default::default(),
                    disks: Default::default(),
                    user_data: "some data".to_string(),
                    external_ips: Vec::from(["mypool".to_string()]),
                    start: true,
                }),

                stdin: "".to_string(),
                want_out: "".to_string(),
                want_err: "-m|--memory required in non-interactive mode".to_string(),
            },
            TestItem {
                name: "delete no --confirm non-interactive".to_string(),
                cmd: crate::cmd_instance::SubCommand::Delete(crate::cmd_instance::CmdInstanceDelete {
                    instance: "things".to_string(),
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
                cmd: crate::cmd_instance::SubCommand::List(crate::cmd_instance::CmdInstanceList {
                    limit: 0,
                    organization: "".to_string(),
                    project: "".to_string(),
                    paginate: false,
                    format: None,
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

            let cmd_instance = crate::cmd_instance::CmdInstance { subcmd: t.cmd };
            match cmd_instance.run(&mut ctx).await {
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
