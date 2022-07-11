use std::str::FromStr;

use anyhow::{anyhow, Result};

use crate::{config::Config, config_file::get_env_var, types::FormatOutput};

pub struct Context<'a> {
    pub config: &'a mut (dyn Config + Send + Sync + 'a),
    pub io: crate::iostreams::IoStreams,
    pub debug: bool,
}

impl Context<'_> {
    pub fn new(config: &mut (dyn Config + Send + Sync)) -> Context {
        // Let's get our IO streams.
        let mut io = crate::iostreams::IoStreams::system();

        // Set the prompt.
        let prompt = config.get("", "prompt").unwrap();
        if prompt == "disabled" {
            io.set_never_prompt(true)
        }

        // Set the pager.
        // Pager precedence
        // 1. OXIDE_PAGER
        // 2. pager from config
        // 3. PAGER
        if let Ok(oxide_pager) = std::env::var("OXIDE_PAGER") {
            io.set_pager(oxide_pager);
        } else if let Ok(pager) = config.get("", "pager") {
            if !pager.is_empty() {
                io.set_pager(pager);
            }
        }

        // Check if we should force use the tty.
        if let Ok(oxide_force_tty) = std::env::var("OXIDE_FORCE_TTY") {
            if !oxide_force_tty.is_empty() {
                io.force_terminal(&oxide_force_tty);
            }
        }

        Context {
            config,
            io,
            debug: false,
        }
    }

    /// This function returns an API client for Oxide that is based on the configured
    /// user.
    pub fn api_client(&self, hostname: &str) -> Result<oxide_api::Client> {
        // Use the host passed in if it's set.
        // Otherwise, use the default host.
        let host = if hostname.is_empty() {
            dbg!(self.config.default_host()?)
        } else {
            hostname.to_string()
        };

        // Change the baseURL to the one we want.
        let mut baseurl = host.to_string();
        if !host.starts_with("http://") && !host.starts_with("https://") {
            baseurl = format!("https://{}", host);
            if host.starts_with("localhost") {
                baseurl = format!("http://{}", host)
            }
        }

        // Get the token for that host.
        let token = self.config.get(&host, "token")?;

        // Create the client.
        let client = oxide_api::Client::new(&token, &baseurl);

        Ok(client)
    }

    /// This function opens a browser that is based on the configured
    /// environment to the specified path.
    ///
    /// Browser precedence:
    /// 1. OXIDE_BROWSER
    /// 2. BROWSER
    /// 3. browser from config
    pub fn browser(&self, hostname: &str, url: &str) -> Result<()> {
        let source: String;
        let browser = if !get_env_var("OXIDE_BROWSER").is_empty() {
            source = "OXIDE_BROWSER".to_string();
            get_env_var("OXIDE_BROWSER")
        } else if !get_env_var("BROWSER").is_empty() {
            source = "BROWSER".to_string();
            get_env_var("BROWSER")
        } else {
            source = crate::config_file::config_file()?;
            self.config.get(hostname, "browser").unwrap_or_else(|_| "".to_string())
        };

        if browser.is_empty() {
            if let Err(err) = open::that(url) {
                return Err(anyhow!("An error occurred when opening '{}': {}", url, err));
            }
        } else if let Err(err) = open::with(url, &browser) {
            return Err(anyhow!(
                "An error occurred when opening '{}' with browser '{}' configured from '{}': {}",
                url,
                browser,
                source,
                err
            ));
        }

        Ok(())
    }

    /// Return the configured output format or override the default with the value passed in,
    /// if it is some.
    pub fn format(&self, format: &Option<FormatOutput>) -> Result<FormatOutput> {
        if let Some(format) = format {
            Ok(format.clone())
        } else {
            let value = self.config.get("", "format")?;
            Ok(FormatOutput::from_str(&value).unwrap_or_default())
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use test_context::{test_context, TestContext};

    use super::*;

    struct TContext {
        orig_oxide_pager_env: Result<String, std::env::VarError>,
        orig_oxide_force_tty_env: Result<String, std::env::VarError>,
    }

    impl TestContext for TContext {
        fn setup() -> TContext {
            TContext {
                orig_oxide_pager_env: std::env::var("OXIDE_PAGER"),
                orig_oxide_force_tty_env: std::env::var("OXIDE_FORCE_TTY"),
            }
        }

        fn teardown(self) {
            // Put the original env var back.
            if let Ok(ref val) = self.orig_oxide_pager_env {
                std::env::set_var("OXIDE_PAGER", val);
            } else {
                std::env::remove_var("OXIDE_PAGER");
            }

            if let Ok(ref val) = self.orig_oxide_force_tty_env {
                std::env::set_var("OXIDE_FORCE_TTY", val);
            } else {
                std::env::remove_var("OXIDE_FORCE_TTY");
            }
        }
    }

    pub struct TestItem {
        name: String,
        oxide_pager_env: String,
        oxide_force_tty_env: String,
        pager: String,
        prompt: String,
        want_pager: String,
        want_prompt: String,
        want_terminal_width_override: i32,
    }

    #[test_context(TContext)]
    #[test]
    #[serial_test::serial]
    fn test_context() {
        let tests = vec![
            TestItem {
                name: "OXIDE_PAGER env".to_string(),
                oxide_pager_env: "more".to_string(),
                oxide_force_tty_env: "".to_string(),
                prompt: "".to_string(),
                pager: "".to_string(),
                want_pager: "more".to_string(),
                want_prompt: "enabled".to_string(),
                want_terminal_width_override: 0,
            },
            TestItem {
                name: "OXIDE_PAGER env override".to_string(),
                oxide_pager_env: "more".to_string(),
                oxide_force_tty_env: "".to_string(),
                prompt: "".to_string(),
                pager: "less".to_string(),
                want_pager: "more".to_string(),
                want_prompt: "enabled".to_string(),
                want_terminal_width_override: 0,
            },
            TestItem {
                name: "config pager".to_string(),
                oxide_pager_env: "".to_string(),
                oxide_force_tty_env: "".to_string(),
                prompt: "".to_string(),
                pager: "less".to_string(),
                want_pager: "less".to_string(),
                want_prompt: "enabled".to_string(),
                want_terminal_width_override: 0,
            },
            TestItem {
                name: "config prompt".to_string(),
                oxide_pager_env: "".to_string(),
                oxide_force_tty_env: "".to_string(),
                prompt: "disabled".to_string(),
                pager: "less".to_string(),
                want_pager: "less".to_string(),
                want_prompt: "disabled".to_string(),
                want_terminal_width_override: 0,
            },
            TestItem {
                name: "OXIDE_FORCE_TTY env".to_string(),
                oxide_pager_env: "".to_string(),
                oxide_force_tty_env: "120".to_string(),
                prompt: "disabled".to_string(),
                pager: "less".to_string(),
                want_pager: "less".to_string(),
                want_prompt: "disabled".to_string(),
                want_terminal_width_override: 120,
            },
        ];

        for t in tests {
            let mut config = crate::config::new_blank_config().unwrap();
            let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

            if !t.pager.is_empty() {
                c.set("", "pager", &t.pager).unwrap();
            }

            if !t.prompt.is_empty() {
                c.set("", "prompt", &t.prompt).unwrap();
            }

            if !t.oxide_pager_env.is_empty() {
                std::env::set_var("OXIDE_PAGER", t.oxide_pager_env.clone());
            } else {
                std::env::remove_var("OXIDE_PAGER");
            }

            if !t.oxide_force_tty_env.is_empty() {
                std::env::set_var("OXIDE_FORCE_TTY", t.oxide_force_tty_env.clone());
            } else {
                std::env::remove_var("OXIDE_FORCE_TTY");
            }

            let ctx = Context::new(&mut c);

            assert_eq!(ctx.io.get_pager(), t.want_pager, "test: {}", t.name);

            assert_eq!(
                ctx.io.get_never_prompt(),
                t.want_prompt == "disabled",
                "test {}",
                t.name
            );

            assert_eq!(ctx.config.get("", "pager").unwrap(), t.want_pager, "test: {}", t.name);
            assert_eq!(ctx.config.get("", "prompt").unwrap(), t.want_prompt, "test: {}", t.name);

            if t.want_terminal_width_override > 0 {
                assert_eq!(
                    ctx.io.terminal_width(),
                    t.want_terminal_width_override,
                    "test: {}",
                    t.name
                );
            }
        }
    }
}
