use std::io::Write;

use clap::{AppSettings, ArgSettings};
use roff::{bold, escape, italic, list, paragraph, ManSection, Roff, Troffable};

/// Man page generator
pub struct Man {
    section: Option<ManSection>,
    manual: Option<String>,
    sections: Vec<(String, String)>,
}

impl Default for Man {
    fn default() -> Self {
        Self {
            section: Some(ManSection::Executable),
            manual: Some("General Commands Manual".to_string()),
            sections: Vec::new(),
        }
    }
}

/// Generate manpage for your application using the most common default values.
pub fn generate_manpage<'a>(app: &mut clap::App<'a>, buf: &mut dyn Write) {
    let man = Man::default();
    man.render(app, buf);
}

impl Man {
    /// Write the manpage to a buffer.
    pub fn render(self, app: &mut clap::App, buf: &mut dyn std::io::Write) {
        app._build_all();

        let mut page = Roff::new(app.get_name(), self.get_section())
            .source(&format!("{} {}", app.get_name(), app.get_version().unwrap_or_default()))
            .section("Name", [&about(app)])
            .section("Synopsis", [&synopsis(app)])
            .section("Description", &description(app));

        if let Some(manual) = &self.manual {
            page = page.manual(manual);
        }

        if app_has_arguments(app) {
            page = page.section("Options", &options(app));
        }

        if app_has_subcommands(app) {
            page = page.section(&subcommand_heading(app), &subcommands(app, self.get_section().value()))
        }

        if app.get_after_long_help().is_some() || app.get_after_help().is_some() {
            page = page.section("Extra", &after_help(app))
        }

        for (title, section) in self.sections {
            page = page.section(&title, &[section]);
        }

        if app_has_version(app) {
            page = page.section("Version", &[version(app)]);
        }

        if app.get_author().is_some() {
            page = page.section("Author(s)", &[app.get_author().unwrap_or_default()]);
        }

        buf.write_all(page.render().as_bytes()).unwrap();
    }

    fn get_section(&self) -> ManSection {
        self.section.unwrap_or(ManSection::Executable)
    }
}

fn app_has_version(app: &clap::App) -> bool {
    app.get_long_version().or_else(|| app.get_version()).is_some()
}

fn app_has_arguments(app: &clap::App) -> bool {
    app.get_arguments().any(|i| !i.is_set(clap::ArgSettings::Hidden))
}

fn app_has_subcommands(app: &clap::App) -> bool {
    app.get_subcommands().any(|i| !i.is_set(clap::AppSettings::Hidden))
}

fn subcommand_heading(app: &clap::App) -> String {
    match app.get_subommand_help_heading() {
        Some(title) => title.to_string(),
        None => "Subcommands".to_string(),
    }
}

fn about(app: &clap::App) -> String {
    match app.get_about().or_else(|| app.get_long_about()) {
        Some(about) => format!("{} - {}", app.get_name(), about),
        None => app.get_name().to_string(),
    }
}

fn description(app: &clap::App) -> Vec<String> {
    match app.get_long_about().or_else(|| app.get_about()) {
        Some(about) => about
            .lines()
            .filter_map(|l| (!l.trim().is_empty()).then(|| paragraph(l.trim())))
            .collect(),
        None => Vec::new(),
    }
}

fn synopsis(app: &clap::App) -> String {
    let mut res = String::new();

    res.push_str(&italic(app.get_name()));
    res.push(' ');

    for opt in app.get_arguments() {
        let (lhs, rhs) = option_markers(opt);
        res.push_str(&match (opt.get_short(), opt.get_long()) {
            (Some(short), Some(long)) => format!("{}-{}|--{}{} ", lhs, short, long, rhs),
            (Some(short), None) => format!("{}-{}{} ", lhs, short, rhs),
            (None, Some(long)) => format!("{}--{}{} ", lhs, long, rhs),
            (None, None) => "".to_string(),
        });
    }

    for arg in app.get_positionals() {
        let (lhs, rhs) = option_markers(arg);
        res.push_str(&format!("{}{}{} ", lhs, arg.get_name(), rhs));
    }

    if app.has_subcommands() {
        let (lhs, rhs) = subcommand_markers(app);
        res.push_str(&format!(
            "{}{}{} ",
            lhs,
            escape(
                &app.get_subcommand_value_name()
                    .unwrap_or(&subcommand_heading(app))
                    .to_lowercase()
            ),
            rhs
        ));
    }

    res
}

fn options(app: &clap::App) -> Vec<String> {
    let mut res = Vec::new();
    let items: Vec<_> = app.get_arguments().filter(|i| !i.is_set(ArgSettings::Hidden)).collect();

    for opt in items.iter().filter(|a| !a.is_positional()) {
        let mut body = Vec::new();

        let mut header = match (opt.get_short(), opt.get_long()) {
            (Some(short), Some(long)) => {
                vec![short_option(short), ", ".to_string(), long_option(long)]
            }
            (Some(short), None) => vec![short_option(short)],
            (None, Some(long)) => vec![long_option(long)],
            (None, None) => vec![],
        };

        if let Some(value) = &opt.get_value_names() {
            header.push(format!("={}", italic(&value.join(" "))));
        }

        if let Some(defs) = option_default_values(opt) {
            header.push(format!(" {}", defs));
        }

        if let Some(help) = opt.get_long_help().or_else(|| opt.get_help()) {
            body.push(help.to_string());
        }

        if let Some(env) = option_environment(opt) {
            body.push(env);
        }

        body.push("\n".to_string());

        res.push(list(&header, &body));
    }

    for pos in items.iter().filter(|a| a.is_positional()) {
        let (lhs, rhs) = option_markers(pos);
        let name = format!("{}{}{}", lhs, pos.get_name(), rhs);

        let mut header = vec![bold(&name)];

        let mut body = Vec::new();

        if let Some(defs) = option_default_values(pos) {
            header.push(format!(" {}", defs));
        }

        if let Some(help) = pos.get_long_help().or_else(|| pos.get_help()) {
            body.push(help.to_string());
        }

        if let Some(env) = option_environment(pos) {
            body.push(env);
        }

        res.push(list(&header, &body))
    }

    res
}

fn subcommands(app: &clap::App, section: i8) -> Vec<String> {
    app.get_subcommands()
        .filter(|s| !s.is_set(AppSettings::Hidden))
        .map(|command| {
            let name = format!("{}-{}({})", app.get_name(), command.get_name(), section);

            let mut body = match command.get_about().or_else(|| command.get_long_about()) {
                Some(about) => about
                    .lines()
                    .filter_map(|l| (!l.trim().is_empty()).then(|| l.trim()))
                    .collect(),
                None => Vec::new(),
            };

            body.push("\n");

            list(&[bold(&name)], &body)
        })
        .collect()
}

fn version(app: &clap::App) -> String {
    format!("v{}", app.get_long_version().or_else(|| app.get_version()).unwrap())
}

fn after_help(app: &clap::App) -> Vec<String> {
    match app.get_after_long_help().or_else(|| app.get_after_help()) {
        Some(about) => about
            .lines()
            .filter_map(|l| (!l.trim().is_empty()).then(|| paragraph(l.trim())))
            .collect(),
        None => Vec::new(),
    }
}

fn subcommand_markers(cmd: &clap::App) -> (&'static str, &'static str) {
    markers(cmd.is_set(AppSettings::SubcommandRequired) || cmd.is_set(AppSettings::SubcommandRequiredElseHelp))
}

fn option_markers(opt: &clap::Arg) -> (&'static str, &'static str) {
    markers(opt.is_set(ArgSettings::Required))
}

fn markers(required: bool) -> (&'static str, &'static str) {
    if required {
        ("<", ">")
    } else {
        ("[", "]")
    }
}

fn short_option(opt: char) -> String {
    format!("-{}", bold(&opt.to_string()))
}

fn long_option(opt: &str) -> String {
    format!("--{}", bold(opt))
}

fn option_environment(opt: &clap::Arg) -> Option<String> {
    if opt.is_set(ArgSettings::HideEnv) {
        return None;
    } else if let Some(env) = opt.get_env() {
        return Some(paragraph(&format!(
            "May also be specified with the {} environment variable. ",
            bold(&env.to_string_lossy())
        )));
    }

    None
}

fn option_default_values(opt: &clap::Arg) -> Option<String> {
    if !opt.get_default_values().is_empty() {
        let values = opt
            .get_default_values()
            .iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(",");

        return Some(format!("[default: {}]", values));
    }

    None
}
