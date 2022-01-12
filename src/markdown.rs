use anyhow::Result;
use clap::App;
use pulldown_cmark_to_cmark::cmark;

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

    fn link_in_list(&mut self, text: String, url: String) {
        let link = pulldown_cmark::Tag::Link(pulldown_cmark::LinkType::Inline, url.into(), text.into());

        self.0.push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item));
        self.0.push(pulldown_cmark::Event::Start(link.clone()));
        self.0.push(pulldown_cmark::Event::End(link));
        self.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::Item));
    }
}

fn do_markdown(doc: &mut MarkdownDocument, app: &App, title: &str) {
    // We don't need the header since our renderer will do that for us.
    //doc.header(app.get_name().to_string(), pulldown_cmark::HeadingLevel::H2);

    if let Some(about) = app.get_about() {
        doc.paragraph(about.to_string());
    }

    if app.has_subcommands() {
        doc.header("Subcommands".to_string(), pulldown_cmark::HeadingLevel::H3);

        doc.0
            .push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(None)));

        for cmd in app.get_subcommands() {
            doc.link_in_list(
                format!("{} {}", title, cmd.get_name()),
                format!("./{}_{}", title.replace(' ', "_"), cmd.get_name()),
            );
        }

        doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::List(None)));
    }

    let args = app.get_arguments().collect::<Vec<&clap::Arg>>();
    if !args.is_empty() {
        doc.header("Options".to_string(), pulldown_cmark::HeadingLevel::H3);

        let mut html = "<dl class=\"flags\">\n".to_string();

        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                html.push('\n');
            }
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

            html.push_str(&format!(
                r#"   <dt><code>{}</code></dt>
   <dd>{}</dd>
"#,
                def,
                arg.get_help().unwrap_or_default()
            ));
        }

        html.push_str("</dl>\n\n");

        doc.0.push(pulldown_cmark::Event::Html(html.into()));
    }

    // TODO: add examples

    if let Some(about) = app.get_long_about() {
        doc.header("About".to_string(), pulldown_cmark::HeadingLevel::H3);

        doc.paragraph(
            about
                .to_string()
                .trim_start_matches(app.get_about().unwrap_or_default())
                .trim_start_matches('.')
                .trim()
                .to_string(),
        );
    }

    // Check if the command has a parent.
    if !(title == app.get_name() || title.trim_start_matches("oxide ") == app.get_name()) {
        doc.header("See also".to_string(), pulldown_cmark::HeadingLevel::H3);

        doc.0
            .push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(None)));

        // Get the parent command.
        // TODO: iterate if more than one, thats why we have a list.
        let parent = title.trim_end_matches(app.get_name()).trim();
        doc.link_in_list(parent.into(), format!("./{}", parent.replace(' ', "_")));
    }
}

/// Convert a clap App to markdown documentation.
pub fn app_to_markdown(app: &App, title: &str) -> Result<String> {
    let mut document = MarkdownDocument(Vec::new());

    do_markdown(&mut document, app, title);

    let mut result = String::new();
    cmark(document.0.iter(), &mut result, None)?;

    Ok(result)
}