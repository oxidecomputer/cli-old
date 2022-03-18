#[derive(Parser, Debug, Clone)]
enum SubCommand {
    List(CmdSubnetList),
    View(CmdSubnetView),
    Delete(CmdSubnetDelete),
}

#[doc = "List subnets."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetList {
    #[doc = "The project that holds the subnets."]
    #[clap(long, short, required = true)]
    pub project: String,
    #[doc = r" The organization that holds the project."]
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
    #[doc = "The order in which to sort the results."]
    #[clap(long, short, default_value_t)]
    pub sort_by: oxide_api::types::NameSortMode,
    #[doc = "The VPC that holds the subnets."]
    #[clap(long, short, required = true)]
    pub vpc: String,
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
impl crate::cmd::Command for CmdSubnetList {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if self.limit < 1 {
            return Err(anyhow::anyhow!("--limit must be greater than 0"));
        }

        let client = ctx.api_client("")?;
        let results = if self.paginate {
            client
                .subnets()
                .get_all(
                    &self.organization,
                    &self.project,
                    self.sort_by.clone(),
                    &self.vpc,
                )
                .await?
        } else {
            client
                .subnets()
                .get_page(
                    self.limit,
                    &self.organization,
                    "",
                    &self.project,
                    self.sort_by.clone(),
                    &self.vpc,
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

#[doc = "View subnet."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetView {
    #[doc = "The project that holds the subnet."]
    #[clap(long, short, required = true)]
    pub project: String,
    #[doc = r" The organization that holds the project."]
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
    #[doc = "The VPC that holds the subnet."]
    #[clap(long, short, required = true)]
    pub vpc: String,
    #[doc = "Open the subnet in the browser."]
    #[clap(short, long)]
    pub web: bool,
    #[doc = r" Output JSON."]
    #[clap(long)]
    pub json: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnetView {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if self.web {
            let url = format!("https://{}/{}", ctx.config.default_host()?, self.subnet);
            ctx.browser("", &url)?;
            return Ok(());
        }

        let client = ctx.api_client("")?;
        let result = client
            .subnets()
            .get(&self.organization, &self.project, &self.subnet, &self.vpc)
            .await?;
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

#[doc = "Delete subnet."]
#[derive(clap :: Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdSubnetDelete {
    #[doc = "The subnet to delete. Can be an ID or name."]
    #[clap(name = "subnet", required = true)]
    pub subnet: String,
    #[doc = "The project to delete the subnet from."]
    #[clap(long, short, required = true)]
    pub project: String,
    #[doc = r" The organization that holds the project."]
    #[clap(long, short, required = true, env = "OXIDE_ORG")]
    pub organization: String,
    #[doc = "The VPC that holds the subnet."]
    #[clap(long, short, required = true)]
    pub vpc: String,
    #[doc = r" Confirm deletion without prompting."]
    #[clap(long)]
    pub confirm: bool,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdSubnetDelete {
    async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
        if !ctx.io.can_prompt() && !self.confirm {
            return Err(anyhow::anyhow!(
                "--confirm required when not running interactively"
            ));
        }

        let client = ctx.api_client("")?;
        if !self.confirm {
            if let Err(err) = dialoguer::Input::<String>::new()
                .with_prompt(format!("Type {} to confirm deletion:", self.subnet))
                .validate_with(|input: &String| -> Result<(), &str> {
                    if input.trim() == self.subnet {
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
            .subnets()
            .delete(&self.organization, &self.project, &self.subnet, &self.vpc)
            .await?;
        let cs = ctx.io.color_scheme();
        let full_name = format!("{}/{}", self.organization, self.project);
        writeln!(
            ctx.io.out,
            "{} Deleted {} {} from {}",
            cs.success_icon_with_color(ansi_term::Color::Red),
            "subnet",
            self.subnet,
            full_name
        )?;
        Ok(())
    }
}
