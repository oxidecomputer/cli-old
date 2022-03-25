use anyhow::Result;
use clap::Parser;

/// Shortcut to open the Oxide documentation or Console in your browser..
///
/// If no arguments are given, the default is to open the Oxide documentation.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdOpen {
    #[clap(name = "shortcut", default_value_t)]
    shortcut: OpenShortcut,
}

/// The type of shortcut to open.
#[derive(PartialEq, Debug, Clone)]
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

impl std::fmt::Display for OpenShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpenShortcut::Docs => write!(f, "https://docs.oxide.computer"),
            OpenShortcut::ApiRef => write!(f, "https://docs.oxide.computer/api"),
            OpenShortcut::CliRef => write!(f, "https://docs.oxide.computer/cli"),
            OpenShortcut::Console => write!(f, ""),
        }
    }
}

impl std::str::FromStr for OpenShortcut {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "docs" => Ok(OpenShortcut::Docs),
            "api-ref" => Ok(OpenShortcut::ApiRef),
            "cli-ref" => Ok(OpenShortcut::CliRef),
            "console" => Ok(OpenShortcut::Console),
            _ => Err(anyhow::anyhow!("Invalid value for [shortcut]: {}", s)),
        }
    }
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdOpen {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        if self.shortcut != OpenShortcut::Console {
            return ctx.browser("", &self.shortcut.to_string());
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
