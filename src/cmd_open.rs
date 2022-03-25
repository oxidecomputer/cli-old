use anyhow::Result;
use clap::Parser;
use parse_display::{Display, FromStr};

/// Shortcut to open the Oxide documentation or Console in your browser.
///
/// If no arguments are given, the default is to open the Oxide documentation.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdOpen {
    #[clap(name = "shortcut", default_value_t)]
    shortcut: OpenShortcut,
}

/// The type of shortcut to open.
#[derive(PartialEq, Debug, Clone, FromStr, Display)]
#[display(style = "kebab-case")]
pub enum OpenShortcut {
    /// Open the Oxide documentation in your browser.
    Docs,
    /// Open the Oxide API reference in your browser.
    ApiRef,
    /// Open the Oxide CLI reference in your browser.
    CliRef,
    /// Open the Oxide Console in your browser.
    Console,
}

impl Default for OpenShortcut {
    fn default() -> Self {
        OpenShortcut::Docs
    }
}

impl OpenShortcut {
    fn get_url(&self) -> String {
        match self {
            OpenShortcut::Docs => "https://docs.oxide.computer".to_string(),
            OpenShortcut::ApiRef => "https://docs.oxide.computer/api".to_string(),
            OpenShortcut::CliRef => "https://docs.oxide.computer/cli".to_string(),
            OpenShortcut::Console => "".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdOpen {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.shortcut != OpenShortcut::Console {
            return ctx.browser("", &self.shortcut.get_url());
        }

        // If they want to open the console, we need to get their default host.
        let mut host = ctx.config.default_host()?;

        if !host.starts_with("http") {
            // Default to https://
            host = format!("https://{}", host);
        }

        // TODO: check this works once we have a proper console.
        ctx.browser("", &host)?;

        Ok(())
    }
}

/// Returns the URL to the changelog for the given version.
pub fn changelog_url(version: &str) -> String {
    format!("https://github.com/oxidecomputer/cli/releases/tag/v{}", version)
}
