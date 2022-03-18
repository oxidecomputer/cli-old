#[derive(Parser, Debug, Clone)]
enum SubCommand {
    List(CmdOrganizationList),
    View(CmdOrganizationView),
    Delete(CmdOrganizationDelete),
}

#[doc = "List organizations."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdOrganizationList {
    #[doc = "The order in which to sort the results."]
    #[clap(long, short, default_value_t)]
    pub sort_by: oxide_api::types::NameOrIdSortMode,
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
impl crate::cmd::Command for CmdOrganizationList {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if self.limit < 1 {
            return Err(anyhow::anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;
        let results = if self.paginate {
            client.organizations().get_all(self.sort_by.clone()).await?
        } else {
            client
                .organizations()
                .get_page(self.limit, "", self.sort_by.clone())
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

#[doc = "View organization."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdOrganizationView {
    #[doc = "Open the organization in the browser."]
    #[clap(short, long)]
    pub web: bool,
    #[doc = r" Output JSON."]
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdOrganizationView {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if self.web {
            let url = format!(
                "https://{}/{}",
                ctx.config.default_host()?,
                self.organization
            );
            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;
        let result = client.organizations().get(&self.organization).await?;
        if self.json {
            ctx.io.write_json(&serde_json::json!(result))?;
            return Ok(());
        }

        let table = tabled::Table::new(results)
            .with(Rotate::Left)
            .with(tabled::Style::psql())
            .to_string();
        writeln!(ctx.io.out, "{}", table)?;
        Ok(())
    }
}

#[doc = "Delete organization."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdOrganizationDelete {
    #[doc = "The organization to delete. Can be an ID or name."]
    #[clap(name = "organization", required = true)]
    pub organization: String,
    #[doc = r" Confirm deletion without prompting."]
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdOrganizationDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow::anyhow!(
                "--confirm required when not running interactively"
            ));
        }

        let client = ctx.api_client("")?;
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.organization))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == self.organization {
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

        client.organizations().delete(&self.organization).await?;
        let cs = ctx.io.color_scheme();
        writeln!(
            ctx.io.out,
            "{} Deleted {} {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            "organization",
            self.organization
        )?;
        Ok(())
    }
}
