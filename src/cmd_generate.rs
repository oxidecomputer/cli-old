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
        let mut app: App = crate::Opts::into_app();
        app._build_all();

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
        writeln!(ctx.io.out, "Generating markdown for `{}` -> {}", title, filename)?;

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
        let mut app: App = crate::Opts::into_app();
        app._build_all();

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
        let mut p = parent.to_string();
        if !p.is_empty() {
            p = format!("{}-{}", p, app.get_name());
        }

        let filename = format!("{}.1", p);
        writeln!(
            ctx.io.out,
            "Generating man page for `{}` -> {}",
            p.replace('-', " "),
            filename
        )?;

        if self.dir.is_empty() {
            crate::man::generate_manpage(app, &mut ctx.io.out);
        } else {
            let p = std::path::Path::new(&self.dir).join(filename);
            let mut file = std::fs::File::create(p)?;
            crate::man::generate_manpage(app, &mut file);
        }

        // Iterate over all the subcommands and generate the documentation.
        for subcmd in app.get_subcommands() {
            // Make it recursive.
            self.generate(ctx, subcmd, &p)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use clap::{arg, App, AppSettings};
    use pretty_assertions::assert_eq;

    use crate::cmd::Command;

    #[test]
    fn test_generate_markdown() {
        let mut config = crate::config::new_blank_config().unwrap();
        let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

        let (io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
        let mut ctx = crate::context::Context { config: &mut c, io };

        let cmd = crate::cmd_generate::CmdGenerateMarkdown { dir: "".to_string() };

        cmd.run(&mut ctx).unwrap();

        let stdout = std::fs::read_to_string(stdout_path).unwrap();
        let stderr = std::fs::read_to_string(stderr_path).unwrap();

        assert!(stdout.contains("<dt><code>-H/--host</code></dt>"), "");
        assert!(stdout.contains("### About"), "");

        assert_eq!(stderr, "");
    }

    #[test]
    fn test_generate_markdown_sub_subcommands() {
        let mut config = crate::config::new_blank_config().unwrap();
        let mut c = crate::config_from_env::EnvConfig::inherit_env(&mut config);

        let (io, stdout_path, stderr_path) = crate::iostreams::IoStreams::test();
        let mut ctx = crate::context::Context { config: &mut c, io };

        let cmd = crate::cmd_generate::CmdGenerateMarkdown { dir: "".to_string() };

        // Define our app.
        let app = App::new("git")
            .about("A fictional versioning CLI")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::AllowExternalSubcommands)
            .setting(AppSettings::AllowInvalidUtf8ForExternalSubcommands)
            .subcommand(
                App::new("clone")
                    .about("Clones repos")
                    .arg(arg!(<REMOTE> "The remote to clone"))
                    .setting(AppSettings::ArgRequiredElseHelp),
            )
            .subcommand(
                App::new("push")
                    .about("pushes things")
                    .arg(arg!(<REMOTE> "The remote to target"))
                    .setting(AppSettings::ArgRequiredElseHelp),
            )
            .subcommand(
                App::new("add")
                    .about("adds things")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .arg(arg!(<PATH> ... "Stuff to add").allow_invalid_utf8(true))
                    .subcommand(
                        App::new("new")
                            .about("subcommand for adding new stuff")
                            .subcommand(App::new("foo").about("sub subcommand")),
                    ),
            );

        cmd.generate(&mut ctx, &app, "").unwrap();

        let expected = r#"Generating markdown for `git` -> git.md
---
title: "git"
description: "A fictional versioning CLI"
layout: manual
---

A fictional versioning CLI

### Subcommands

* [git clone](./git_clone)
* [git push](./git_push)
* [git add](./git_add)

### Options

<dl class="flags">
   <dt><code>--help</code></dt>
   <dd>Print help information</dd>

   <dt><code>--version</code></dt>
   <dd>Print version information</dd>
</dl>


Generating markdown for `git clone` -> git_clone.md
---
title: "git clone"
description: "Clones repos"
layout: manual
---

Clones repos

### Options

<dl class="flags">
   <dt><code>--help</code></dt>
   <dd>Print help information</dd>

   <dt><code>--version</code></dt>
   <dd>Print version information</dd>

   <dt><code></code></dt>
   <dd>The remote to clone</dd>
</dl>


Generating markdown for `git push` -> git_push.md
---
title: "git push"
description: "pushes things"
layout: manual
---

pushes things

### Options

<dl class="flags">
   <dt><code>--help</code></dt>
   <dd>Print help information</dd>

   <dt><code>--version</code></dt>
   <dd>Print version information</dd>

   <dt><code></code></dt>
   <dd>The remote to target</dd>
</dl>


Generating markdown for `git add` -> git_add.md
---
title: "git add"
description: "adds things"
layout: manual
---

adds things

### Subcommands

* [git add new](./git_add_new)

### Options

<dl class="flags">
   <dt><code>--help</code></dt>
   <dd>Print help information</dd>

   <dt><code>--version</code></dt>
   <dd>Print version information</dd>

   <dt><code></code></dt>
   <dd>Stuff to add</dd>
</dl>


Generating markdown for `git add new` -> git_add_new.md
---
title: "git add new"
description: "subcommand for adding new stuff"
layout: manual
---

subcommand for adding new stuff

### Subcommands

* [git add new foo](./git_add_new_foo)

### Options

<dl class="flags">
   <dt><code>--help</code></dt>
   <dd>Print help information</dd>

   <dt><code>--version</code></dt>
   <dd>Print version information</dd>
</dl>


### See also

* [git add](./git_add)
Generating markdown for `git add new foo` -> git_add_new_foo.md
---
title: "git add new foo"
description: "sub subcommand"
layout: manual
---

sub subcommand

### Options

<dl class="flags">
   <dt><code>--help</code></dt>
   <dd>Print help information</dd>

   <dt><code>--version</code></dt>
   <dd>Print version information</dd>
</dl>


### See also

* [git add](./git_add)
* [git add new](./git_add_new)
"#;

        let stdout = std::fs::read_to_string(stdout_path).unwrap();
        let stderr = std::fs::read_to_string(stderr_path).unwrap();

        assert_eq!(stdout, expected);
        assert_eq!(stderr, "");
    }
}
