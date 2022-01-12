use std::{fs, io::Write};

use anyhow::{Context, Result};
use clap::{App, IntoApp, Parser};

/// Generate various documentation files for the oxide command line.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdGenerate {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Markdown(CmdGenerateMarkdown),
    ManPages(CmdGenerateManPages),
}

impl crate::cmd::Command for CmdGenerate {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        match &self.subcmd {
            SubCommand::Markdown(cmd) => cmd.run(ctx),
            SubCommand::ManPages(cmd) => cmd.run(ctx),
        }
    }
}

/// Generate markdown documentation.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdGenerateMarkdown {
    /// Path directory where you want to output the generated files.
    #[clap(short = 'D', long, default_value = "")]
    dir: String,
}

impl crate::cmd::Command for CmdGenerateMarkdown {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let app: App = crate::Opts::into_app();

        // Make sure the output directory exists.
        if !self.dir.is_empty() {
            fs::create_dir_all(&self.dir).with_context(|| format!("failed to create directory {}", self.dir))?;
        }

        self.generate(ctx, &app, "")?;

        Ok(())
    }
}

impl CmdGenerateMarkdown {
    fn generate(&self, ctx: &mut crate::context::Context, app: &App, parent: &str) -> Result<()> {
        let mut p = parent.to_string();
        if !p.is_empty() {
            p = format!("{}_{}", p, app.get_name());
        } else {
            p = app.get_name().to_string();
        }

        let filename = format!("{}.md", p);
        let title = p.replace('_', " ");
        println!("Generating markdown for `{}` -> {}", title, filename);

        // Generate the markdown.
        let m = crate::markdown::app_to_markdown(app, &title)?;

        // Add our header information.
        let markdown = format!(
            r#"---
title: "{}"
description: "{}"
layout: manual
---

{}"#,
            title,
            app.get_about().unwrap_or_default(),
            m
        );
        if self.dir.is_empty() {
            // TODO: glamorize markdown to the shell.
            writeln!(ctx.io.out, "{}", markdown)?;
        } else {
            let p = std::path::Path::new(&self.dir).join(filename);
            let mut file = std::fs::File::create(p)?;
            file.write_all(markdown.as_bytes())?;
        }

        // Iterate over all the subcommands and generate the documentation.
        for subcmd in app.get_subcommands() {
            self.generate(ctx, subcmd, &p)?;
        }

        Ok(())
    }
}

/// Generate manual pages.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdGenerateManPages {
    /// Path directory where you want to output the generated files.
    #[clap(short = 'D', long, default_value = "")]
    dir: String,
}

impl crate::cmd::Command for CmdGenerateManPages {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        let app: App = crate::Opts::into_app();

        // Make sure the output directory exists.
        if !self.dir.is_empty() {
            fs::create_dir_all(&self.dir).with_context(|| format!("failed to create directory {}", self.dir))?;
        }

        self.generate(ctx, &app, app.get_name())?;

        Ok(())
    }
}

impl CmdGenerateManPages {
    fn generate(&self, ctx: &mut crate::context::Context, app: &App, parent: &str) -> Result<()> {
        // Iterate over all the subcommands and generate the documentation.
        for s in app.get_subcommands() {
            let mut subcmd = s.clone();
            let mut p = parent.to_string();
            if !p.is_empty() {
                p = format!("{}-{}", p, subcmd.get_name());
            }

            let filename = format!("{}.1", p);
            println!("Generating man page for `{}` -> {}", p.replace('-', " "), filename);

            if self.dir.is_empty() {
                clap_man::generate_manpage(&mut subcmd, &mut ctx.io.out);
            } else {
                let p = std::path::Path::new(&self.dir).join(filename);
                let mut file = std::fs::File::create(p)?;
                clap_man::generate_manpage(&mut subcmd, &mut file);
            }

            // Make it recursive.
            self.generate(ctx, &subcmd, &p)?;
        }

        Ok(())
    }
}
