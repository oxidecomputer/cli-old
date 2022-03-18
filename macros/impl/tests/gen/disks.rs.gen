#[derive(Parser, Debug, Clone)]
enum SubCommand {
    Attach(CmdDiskAttach),
    Create(CmdDiskCreate),
    Detach(CmdDiskDetach),
    Edit(CmdDiskEdit),
    List(CmdDiskList),
    View(CmdDiskView),
    Delete(CmdDiskDelete),
}

#[doc = "List disks."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskList {
    #[doc = "The project that holds the disks."]
    #[clap(long, short, required = true)]
    pub project: String,
    #[doc = r" The organization that holds the project."]
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
    #[doc = "The order in which to sort the results."]
    #[clap(long, short, default_value_t)]
    pub sort_by: oxide_api::types::NameSortMode,
    #[doc = r" Maximum number of items to list."]
    #[clap(long, short, default_value = "30")]
    pub limit: u32,
    #[doc = r" Make additional HTTP requests to fetch all pages."]
    #[clap(long)]
    pub paginate: bool,
    #[doc = r" Output JSON."]
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskList {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if self.limit < 1 {
            return Err(anyhow::anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;
        let results = if self.paginate {
            client
                .disks()
                .get_all(&self.organization, &self.project, self.sort_by.clone())
                .await?
        } else {
            client
                .disks()
                .get_page(
                    self.limit,
                    &self.organization,
                    "",
                    &self.project,
                    self.sort_by.clone(),
                )
                .await?
        };
        if self.json {
            ctx.io.write_json(&serde_json::json!(results))?;
            return Ok(());
        }

        let table = tabled::Table::new(results)
            .with(tabled::Style::psql())
            .to_string();
        writeln!(ctx.io.out, "{}", table)?;
        Ok(())
    }
}

#[doc = "View disk."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskView {
    #[doc = "The disk to view. Can be an ID or name."]
    #[clap(name = "disk", required = true)]
    pub disk: String,
    #[doc = "The project that holds the disk."]
    #[clap(long, short, required = true)]
    pub project: String,
    #[doc = r" The organization that holds the project."]
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
    #[doc = "Open the disk in the browser.\n\nDisplay information about an Oxide disk.\n\nWith '--web', open the disk in a web browser instead.\n            "]
    #[clap(short, long)]
    pub web: bool,
    #[doc = r" Output JSON."]
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskView {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if self.web {
            let url = format!("https://{}/{}", ctx.config.default_host()?, self.disk);
            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;
        let result = client
            .disks()
            .get(&self.disk, &self.organization, &self.project)
            .await?;
        if self.json {
            ctx.io.write_json(&serde_json::json!(result))?;
            return Ok(());
        }

        let table = tabled::Table::new(vec![result])
            .with(tabled::Rotate::Left)
            .with(
                tabled::Modify::new(tabled::Full)
                    .with(tabled::Alignment::left())
                    .with(tabled::Alignment::top()),
            )
            .with(tabled::Style::psql())
            .to_string();
        writeln!(ctx.io.out, "{}", table)?;
        Ok(())
    }
}

#[doc = "Delete disk."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdDiskDelete {
    #[doc = "The disk to delete. Can be an ID or name."]
    #[clap(name = "disk", required = true)]
    pub disk: String,
    #[doc = "The project to delete the disk from."]
    #[clap(long, short, required = true)]
    pub project: String,
    #[doc = r" The organization that holds the project."]
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
    #[doc = r" Confirm deletion without prompting."]
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdDiskDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow::anyhow!(
                "--confirm required when not running interactively"
            ));
        }

        let client = ctx.api_client("")?;
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.disk))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == self.disk {
                        Ok(())
                    } else {
                        Err("mismatched confirmation")
                    }
                })
                .interact_text()
            {
                return Err(anyhow::anyhow!("prompt failed: {}", err));
            }
        }

        client
            .disks()
            .delete(&self.disk, &self.organization, &self.project)
            .await?;
        let cs = ctx.io.color_scheme();
        let full_name = format!("{}/{}", self.organization, self.project);
        writeln!(
            ctx.io.out,
            "{} Deleted {} {} from {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            "disk",
            self.disk,
            full_name
        )?;
        Ok(())
    }
}
