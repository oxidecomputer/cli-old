use std::fs;
use std::io::Write;

use anyhow::{Context, Result};
use clap::{App, IntoApp, Parser};
use pulldown_cmark_to_cmark::cmark;

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

        self.generate(ctx, &app, app.get_name())?;

        Ok(())
    }
}

impl CmdGenerateMarkdown {
    fn generate(&self, ctx: &mut crate::context::Context, app: &App, parent: &str) -> Result<()> {
        // Iterate over all the subcommands and generate the documentation.
        for subcmd in app.get_subcommands() {
            let mut p = parent.to_string();
            if !p.is_empty() {
                p = format!("{}_{}", p, subcmd.get_name());
            }

            let filename = format!("{}.md", p);
            println!("Generating markdown for `{}` -> {}", p.replace('_', " "), filename);

            // Generate the markdown.
            let markdown = app_to_md(app, 2)?;
            if self.dir.is_empty() {
                writeln!(ctx.io.out, "{}", markdown)?;
            } else {
                let p = std::path::Path::new(&self.dir).join(filename);
                let mut file = std::fs::File::create(p)?;
                file.write_all(markdown.as_bytes())?;
            }

            // Make it recursive.
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

struct MarkdownDocument<'a>(Vec<pulldown_cmark::Event<'a>>);

impl MarkdownDocument<'_> {
    fn header(&mut self, text: String, level: pulldown_cmark::HeadingLevel) {
        self.0.push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::Heading(
            level,
            None,
            vec![],
        )));
        self.0.push(pulldown_cmark::Event::Text(text.into()));
        self.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::Heading(
            level,
            None,
            vec![],
        )));
    }

    fn paragraph(&mut self, text: String) {
        self.0
            .push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::Paragraph));
        self.0.push(pulldown_cmark::Event::Text(text.into()));
        self.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::Paragraph));
    }
}

fn to_heading_level(item: i32) -> pulldown_cmark::HeadingLevel {
    match item {
        1 => pulldown_cmark::HeadingLevel::H1,
        2 => pulldown_cmark::HeadingLevel::H2,
        3 => pulldown_cmark::HeadingLevel::H3,
        4 => pulldown_cmark::HeadingLevel::H4,
        5 => pulldown_cmark::HeadingLevel::H5,
        6 => pulldown_cmark::HeadingLevel::H6,
        _ => unreachable!(),
    }
}

fn do_markdown(doc: &mut MarkdownDocument, app: &App, level: i32, skip_header: bool) {
    if !skip_header {
        doc.header(app.get_name().to_string(), to_heading_level(level));
    }

    if let Some(about) = app.get_about() {
        doc.paragraph(about.to_string());
    }

    if let Some(author) = app.get_author() {
        doc.paragraph(format!("Author: {}", author));
    }

    if let Some(version) = app.get_version() {
        doc.paragraph(format!("Version: {}", version));
    }

    let args = app.get_arguments().collect::<Vec<&clap::Arg>>();
    if !args.is_empty() {
        doc.paragraph("Arguments:".to_string());
        doc.0
            .push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(None)));

        for arg in &args {
            doc.0.push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item));
            doc.0.push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::Paragraph));

            doc.0.push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::CodeBlock(
                pulldown_cmark::CodeBlockKind::Indented,
            )));

            let mut def = String::new();
            if let Some(short) = arg.get_short() {
                def.push('-');
                def.push(short);
            }
            if let Some(long) = arg.get_long() {
                if arg.get_short().is_some() {
                    def.push('/');
                }
                def.push_str("--");
                def.push_str(long);
            }

            if arg.is_set(clap::ArgSettings::TakesValue) {
                def.push_str("=<");
                def.push_str(arg.get_name());
                def.push('>');
            }

            doc.0.push(pulldown_cmark::Event::Text(def.into()));
            doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::CodeBlock(
                pulldown_cmark::CodeBlockKind::Indented,
            )));

            let mut text = String::new();
            if let Some(help) = arg.get_help() {
                if arg.get_short().is_some() || arg.get_long().is_some() {
                    text.push_str(": ");
                }
                text.push_str(help);
            }
            doc.0.push(pulldown_cmark::Event::Text(text.into()));

            doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::Paragraph));
            doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::Item));
        }

        doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::List(None)));
    }

    if app.has_subcommands() {
        doc.header("Subcommands".to_string(), to_heading_level(level + 1));

        for cmd in app.get_subcommands() {
            do_markdown(doc, cmd, level + 2, false);
        }
    }
}

/// Convert a clap App to markdown documentation.
///
/// # Parameters:
///
/// - `app`: A reference to a clap application definition
/// - `level`: The level for first markdown headline. If you for example want to
///     render this beneath a `## Usage` headline in your readme, you'd want to
///     set `level` to `2`.
fn app_to_md(app: &App, level: i32) -> Result<String> {
    let mut document = MarkdownDocument(Vec::new());

    do_markdown(&mut document, app, level, level > 1);

    let mut result = String::new();
    cmark(document.0.iter(), &mut result, None)?;

    Ok(result)
}
