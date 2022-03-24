use std::{io::Write, sync::Arc};

use anyhow::Result;
use thrussh::client;

/// A new SSH client that works like ssh(1).
pub struct SshClient {}

impl thrussh::client::Handler for SshClient {
    type Error = anyhow::Error;
    type FutureUnit = futures::future::Ready<Result<(Self, client::Session)>>;
    type FutureBool = futures::future::Ready<Result<(Self, bool)>>;

    fn finished_bool(self, b: bool) -> Self::FutureBool {
        futures::future::ready(Ok((self, b)))
    }

    fn finished(self, session: client::Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, session)))
    }

    fn check_server_key(self, _server_public_key: &thrussh_keys::key::PublicKey) -> Self::FutureBool {
        self.finished_bool(true)
    }

    /*fn channel_open_confirmation(
        self,
        channel: thrushh::ChannelId,
        max_packet_size: u32,
        window_size: u32,
        session: client::Session,
    ) -> Self::FutureUnit {
        self.finished(session)
    }

    fn data(self, channel: thrussh::ChannelId, data: &[u8], session: client::Session) -> Self::FutureUnit {
        self.finished(session)
    }*/
}

pub struct SshSession {
    session: client::Handle<SshClient>,
}

impl SshSession {
    pub async fn connect(
        key: &thrussh_keys::key::KeyPair,
        user: impl Into<String>,
        addr: impl std::net::ToSocketAddrs,
    ) -> Result<Self> {
        let pubkey = key.clone_public_key();

        // Create our SSH client config.
        let config = Arc::new(thrussh::client::Config::default());

        // Create our SSH client that will handle the commands.
        let sh = SshClient {};

        // Create our SSH agent.
        let mut agent = thrussh_keys::agent::client::AgentClient::connect_env().await?;
        agent.add_identity(key, &[]).await?;

        // Start our SSH session.
        let mut session = thrussh::client::connect(config, addr, sh).await?;

        let (_bool, auth_res) = session.authenticate_future(user, pubkey, agent).await;
        let _auth_res = auth_res?;

        Ok(Self { session })
    }

    pub async fn call(&mut self, command: &str) -> Result<SshCommandResult> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;
        let mut output = Vec::new();
        let mut code = None;
        while let Some(msg) = channel.wait().await {
            match msg {
                thrussh::ChannelMsg::Data { ref data } => {
                    output.write_all(data).unwrap();
                }
                thrussh::ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }
                _ => {}
            }
        }
        Ok(SshCommandResult { output, code })
    }

    /// Close the SSH session.
    pub async fn close(&mut self) -> Result<()> {
        self.session
            .disconnect(thrussh::Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

pub struct SshCommandResult {
    output: Vec<u8>,
    code: Option<u32>,
}

impl SshCommandResult {
    pub fn output(&self) -> String {
        String::from_utf8_lossy(&self.output).into()
    }

    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}
