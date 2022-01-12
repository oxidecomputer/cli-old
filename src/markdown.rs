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
            let link = format!(
                r#"[{} {}](./{})"#,
                title,
                cmd.get_name(),
                format!("{}_{}", title.replace(" ", "_"), cmd.get_name())
            );

            doc.0.push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item));
            // TODO: make the link a real link type.
            doc.0.push(pulldown_cmark::Event::Text(link.into()));
            doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::Item));
        }

        doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::List(None)));
    }

    let args = app.get_arguments().collect::<Vec<&clap::Arg>>();
    if !args.is_empty() {
        doc.header("Options".to_string(), pulldown_cmark::HeadingLevel::H3);

        let mut html = "<dl class=\"flags\">\n".to_string();

        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                html.push_str("\n");
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
                arg.get_help().unwrap_or_default().to_string()
            ));
        }

        html.push_str("</dl>\n\n");

        doc.0.push(pulldown_cmark::Event::Html(html.into()));
    }

    // TODO: add examples

    // Check if the command has a parent.
    if !(title == app.get_name() || title.trim_start_matches("oxide ") == app.get_name()) {
        doc.header("See also".to_string(), pulldown_cmark::HeadingLevel::H3);

        doc.0
            .push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::List(None)));

        // Get the parent command.
        // TODO: iterate if more than one, thats why we have a list.
        let parent = title.trim_end_matches(app.get_name()).trim();
        let link = format!(r#"[{}](./{})"#, parent, parent.replace(" ", "_"));

        doc.0.push(pulldown_cmark::Event::Start(pulldown_cmark::Tag::Item));
        // TODO: make the link a real link type.
        doc.0.push(pulldown_cmark::Event::Text(link.into()));
        doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::Item));

        doc.0.push(pulldown_cmark::Event::End(pulldown_cmark::Tag::List(None)));
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
