use std::{mem::swap, os::unix::io::AsRawFd, time::Duration};

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use http::HeaderMap;
use reqwest::ClientBuilder;
use tokio_tungstenite::{
    tungstenite::protocol::{Message, Role},
    WebSocketStream,
};

mod nexus_client {
    progenitor::generate_api!(spec = "spec-serial.json", interface = Builder,);
}

impl super::cmd_instance::CmdInstanceSerial {
    pub(crate) async fn websock_stream_tty(&self, ctx: &mut crate::context::Context<'_>) -> Result<()> {
        // shenanigans to get the info we need to construct a progenitor-client
        let reqw = ctx
            .api_client("")?
            .request_raw(http::Method::GET, "", None)
            .await?
            .build()?;

        let base = reqw.url().as_str();
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::AUTHORIZATION,
            reqw.headers().get(http::header::AUTHORIZATION).unwrap().to_owned(),
        );

        let reqw_client = ClientBuilder::new()
            .connect_timeout(Duration::new(60, 0))
            .default_headers(headers)
            .http1_only() // HTTP2 does not support websockets
            .build()?;

        let nexus_client = nexus_client::Client::new_with_client(base, reqw_client);

        let upgraded = nexus_client
            .instance_serial_console_stream()
            .organization_name(self.organization.to_owned())
            .project_name(self.project.to_owned())
            .instance_name(self.instance.to_owned())
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .into_inner();

        let mut ws = WebSocketStream::from_raw_socket(upgraded, Role::Client, None).await;

        let mut stdin: Box<dyn std::io::Read + Send + Sync> = Box::new(std::io::empty());
        let mut stdout: Box<dyn std::io::Write + Send + Sync> = Box::new(std::io::sink());
        swap(&mut stdin, &mut ctx.io.stdin);
        swap(&mut stdout, &mut ctx.io.out);

        let _raw_guard = if ctx.io.is_stdout_tty() {
            Some(RawTermiosGuard::stdio_guard().expect("failed to set raw mode"))
        } else if cfg!(test) {
            None
        } else {
            return Err(anyhow::anyhow!("Stdout must be a TTY to use interactive mode."));
        };

        // https://docs.rs/tokio/latest/tokio/io/trait.AsyncReadExt.html#method.read_exact
        // is not cancel safe! Meaning reads from tokio::io::stdin are not cancel
        // safe. Spawn a separate task to read and put bytes onto this channel.
        let (stdintx, stdinrx) = tokio::sync::mpsc::channel(16);
        let (wstx, mut wsrx) = tokio::sync::mpsc::channel(16);

        tokio::spawn(async move {
            let mut inbuf = [0u8; 1024];

            loop {
                let n = match tokio::task::block_in_place(|| stdin.read(&mut inbuf)) {
                    Err(_) | Ok(0) => break,
                    Ok(n) => n,
                };

                stdintx.send(inbuf[0..n].to_vec()).await.unwrap();
            }
        });

        tokio::spawn(async move { stdin_to_websockets_task(stdinrx, wstx).await });

        loop {
            tokio::select! {
                c = wsrx.recv() => {
                    match c {
                        None => {
                            // channel is closed
                            break;
                        }
                        Some(c) => {
                            ws.send(Message::Binary(c)).await?;
                        },
                    }
                }
                msg = ws.next() => {
                    match msg {
                        Some(Ok(Message::Binary(input))) => {
                            tokio::task::block_in_place(|| {
                                stdout.write_all(&input)?;
                                stdout.flush()?;
                                Ok::<(), std::io::Error>(())
                            })?;
                        }
                        Some(Ok(Message::Close(..))) | None => break,
                        _ => continue,
                    }
                }
            }
        }

        Ok(())
    }
}

/// Guard object that will set the terminal to raw mode and restore it
/// to its previous state when it's dropped
struct RawTermiosGuard(libc::c_int, libc::termios);

impl RawTermiosGuard {
    fn stdio_guard() -> Result<RawTermiosGuard, std::io::Error> {
        let fd = std::io::stdout().as_raw_fd();
        let termios = unsafe {
            let mut curr_termios = std::mem::zeroed();
            let r = libc::tcgetattr(fd, &mut curr_termios);
            if r == -1 {
                return Err(std::io::Error::last_os_error());
            }
            curr_termios
        };
        let guard = RawTermiosGuard(fd, termios);
        unsafe {
            let mut raw_termios = termios;
            libc::cfmakeraw(&mut raw_termios);
            let r = libc::tcsetattr(fd, libc::TCSAFLUSH, &raw_termios);
            if r == -1 {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(guard)
    }
}

impl Drop for RawTermiosGuard {
    fn drop(&mut self) {
        let r = unsafe { libc::tcsetattr(self.0, libc::TCSADRAIN, &self.1) };
        if r == -1 {
            Err::<(), _>(std::io::Error::last_os_error()).unwrap();
        }
    }
}

async fn stdin_to_websockets_task(
    mut stdinrx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    wstx: tokio::sync::mpsc::Sender<Vec<u8>>,
) {
    // next_raw must live outside loop, because Ctrl-A should work across
    // multiple inbuf reads.
    let mut next_raw = false;

    loop {
        let inbuf = if let Some(inbuf) = stdinrx.recv().await {
            inbuf
        } else {
            continue;
        };

        // Put bytes from inbuf to outbuf, but don't send Ctrl-A unless
        // next_raw is true.
        let mut outbuf = Vec::with_capacity(inbuf.len());

        let mut exit = false;
        for c in inbuf {
            match c {
                // Ctrl-A means send next one raw
                b'\x01' => {
                    if next_raw {
                        // Ctrl-A Ctrl-A should be sent as Ctrl-A
                        outbuf.push(c);
                        next_raw = false;
                    } else {
                        next_raw = true;
                    }
                }
                b'\x03' => {
                    if !next_raw {
                        // Exit on non-raw Ctrl-C
                        exit = true;
                        break;
                    } else {
                        // Otherwise send Ctrl-C
                        outbuf.push(c);
                        next_raw = false;
                    }
                }
                _ => {
                    outbuf.push(c);
                    next_raw = false;
                }
            }
        }

        // Send what we have, even if there's a Ctrl-C at the end.
        if !outbuf.is_empty() {
            wstx.send(outbuf).await.unwrap();
        }

        if exit {
            break;
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use test_context::{test_context, AsyncTestContext};

    use crate::cmd::Command;

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

    // TODO: Auth is shaky with current docker container CI implementation.
    // remove ignore tag once tests run against mock API server
    #[ignore]
    #[test_context(TContext)]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_cmd_instance_serial_interactive() {
        let cmd = crate::cmd_instance::CmdInstanceSerial {
            instance: "things".to_string(),
            project: "bar".to_string(),
            organization: "foo".to_string(),
            max_bytes: None,
            byte_offset: None,
            continuous: false,
            interactive: true,
        };
        let mut config = crate::config::new_blank_config().unwrap();
        let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);
        let (mut io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
        io.stdin = Box::new(std::io::Cursor::new(""));
        let mut ctx = crate::context::Context {
            config: &mut c,
            io,
            debug: false,
        };
        cmd.run(&mut ctx).await.unwrap();

        let stdout = std::fs::read_to_string(stdout_path).unwrap();
        let stderr = std::fs::read_to_string(stderr_path).unwrap();
        assert!(stderr.is_empty());
        assert_eq!(stdout, "");
    }
}
