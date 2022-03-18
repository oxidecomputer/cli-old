use pretty_assertions::assert_eq;

use super::*;

#[test]
fn test_crud_gen() {
    let mut ret = do_gen(
        quote! {
            tag = "disks",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {
                Attach(CmdDiskAttach),
                Create(CmdDiskCreate),
                Detach(CmdDiskDetach),
                Edit(CmdDiskEdit),
                View(CmdDiskView),
            }
        },
    );
    let mut expected = quote! {
        #[derive(Parser, Debug, Clone)]
        enum SubCommand {
            Attach(CmdDiskAttach),
            Create(CmdDiskCreate),
            Detach(CmdDiskDetach),
            Edit(CmdDiskEdit),
            View(CmdDiskView),
            List(CmdDiskList),
            Delete(CmdDiskDelete)
        }

        #[doc = "List disks."]
        #[derive(clap::Parser, Debug, Clone)]
        #[clap(verbatim_doc_comment)]
        pub struct CmdDiskList {
            #[doc = "The project that holds the disks."]
            #[clap(long, short, required = true)]
            pub project: String,

            /// The organization that holds the project.
            #[clap(long, short, required = true, env = "OXIDE_ORG")]
            pub organization: String,

            #[doc = "The order in which to sort the results."]
            #[clap(long, short, default = oxide_api::types::NameSortMode::default())]
            pub sort_by: oxide_api::types::NameSortMode,

            /// Maximum number of items to list.
            #[clap(long, short, default_value = "30")]
            pub limit: u32,

            /// Make additional HTTP requests to fetch all pages.
            #[clap(long)]
            pub paginate: bool,

            /// Output JSON.
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
                        .get_all(
                            &self.organization,
                            &self.project,
                            &self.sort_by
                        )
                        .await?
                } else {
                    client
                        .disks()
                        .get_page(
                            self.limit,
                            "",
                            &self.organization,
                            &self.project,
                            &self.sort_by
                        )
                        .await?
                };

                if self.json {
                    // If they specified --json, just dump the JSON.
                    ctx.io.write_json(&serde_json::json!(results))?;
                    return Ok(());
                }

                let table = tabled::Table::new(results).to_string();
                write!(ctx.io.out, "{}", table)?;

                Ok(())
            }
        }

        #[doc = "Delete disk."]
        #[derive(clap::Parser, Debug, Clone)]
        #[clap(verbatim_doc_comment)]
        pub struct CmdDiskDelete {
            #[doc = "The disk to delete. Can be an ID or name."]
            #[clap(name = "disk", required = true)]
            pub disk: String,

            #[doc = "The project to delete the disk from."]
            #[clap(long, short, required = true)]
            pub project: String,

            /// The organization that holds the project.
            #[clap(long, short, required = true, env = "OXIDE_ORG")]
            pub organization: String,

            /// Confirm deletion without prompting.
            #[clap(long)]
            pub confirm: bool,
        }

        #[async_trait::async_trait]
        impl crate::cmd::Command for CmdDiskDelete {
            async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                if !ctx.io.can_prompt() && !self.confirm {
                    return Err(anyhow::anyhow!("--confirm required when not running interactively"));
                }

                let client = ctx.api_client("")?;

                // Confirm deletion.
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
    };

    assert_eq!(expected.to_string(), ret.unwrap().to_string());

    ret = do_gen(
        quote! {
            tag = "organizations",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    );

    expected = quote! {
        #[derive(Parser, Debug, Clone)]
        enum SubCommand {
            List(CmdOrganizationList),
            Delete(CmdOrganizationDelete)
        }

        #[doc = "List organizations."]
        #[derive(clap::Parser, Debug, Clone)]
        #[clap(verbatim_doc_comment)]
        pub struct CmdOrganizationList {
            #[doc = "The order in which to sort the results."]
            #[clap(long, short, default = oxide_api::types::NameOrIdSortMode::default())]
            pub sort_by: oxide_api::types::NameOrIdSortMode,

            /// Maximum number of items to list.
            #[clap(long, short, default_value = "30")]
            pub limit: u32,

            /// Make additional HTTP requests to fetch all pages.
            #[clap(long)]
            pub paginate: bool,

            /// Output JSON.
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
                    client
                        .organizations()
                        .get_all(&self.sort_by)
                        .await?
                } else {
                    client
                        .organizations()
                        .get_page(
                            self.limit,
                            "",
                            &self.sort_by
                        )
                        .await?
                };

                if self.json {
                    // If they specified --json, just dump the JSON.
                    ctx.io.write_json(&serde_json::json!(results))?;
                    return Ok(());
                }

                let table = tabled::Table::new(results).to_string();
                write!(ctx.io.out, "{}", table)?;

                Ok(())
            }
        }

        #[doc = "Delete organization."]
        #[derive(clap::Parser, Debug, Clone)]
        #[clap(verbatim_doc_comment)]
        pub struct CmdOrganizationDelete {
            #[doc = "The organization to delete. Can be an ID or name."]
            #[clap(name = "organization", required = true)]
            pub organization: String,

            /// Confirm deletion without prompting.
            #[clap(long)]
            pub confirm: bool,
        }

        #[async_trait::async_trait]
        impl crate::cmd::Command for CmdOrganizationDelete {
            async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                if !ctx.io.can_prompt() && !self.confirm {
                    return Err(anyhow::anyhow!("--confirm required when not running interactively"));
                }

                let client = ctx.api_client("")?;

                // Confirm deletion.
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
    };

    assert_eq!(expected.to_string(), ret.unwrap().to_string());

    ret = do_gen(
        quote! {
            tag = "subnets",
        },
        quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {}
        },
    );

    expected = quote! {
        #[derive(Parser, Debug, Clone)]
        enum SubCommand {
            List(CmdSubnetList),
            Delete(CmdSubnetDelete)
        }

        #[doc = "List subnets."]
        #[derive(clap::Parser, Debug, Clone)]
        #[clap(verbatim_doc_comment)]
        pub struct CmdSubnetList {
            #[doc = "The project that holds the subnets."]
            #[clap(long, short, required = true)]
            pub project: String,

            /// The organization that holds the project.
            #[clap(long, short, required = true, env = "OXIDE_ORG")]
            pub organization: String,

            #[doc = "The order in which to sort the results."]
            #[clap(long, short, default = oxide_api::types::NameSortMode::default())]
            pub sort_by: oxide_api::types::NameSortMode,

            #[doc = "The VPC that holds the subnets."]
            #[clap(long, short, required = true)]
            pub vpc: String,

            /// Maximum number of items to list.
            #[clap(long, short, default_value = "30")]
            pub limit: u32,

            /// Make additional HTTP requests to fetch all pages.
            #[clap(long)]
            pub paginate: bool,

            /// Output JSON.
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
                            &self.sort_by,
                            &self.vpc
                        )
                        .await?
                } else {
                    client
                        .subnets()
                        .get_page(
                            self.limit,
                            "",
                            &self.organization,
                            &self.project,
                            &self.sort_by,
                            &self.vpc
                        )
                        .await?
                };

                if self.json {
                    // If they specified --json, just dump the JSON.
                    ctx.io.write_json(&serde_json::json!(results))?;
                    return Ok(());
                }

                let table = tabled::Table::new(results).to_string();
                write!(ctx.io.out, "{}", table)?;

                Ok(())
            }
        }

        #[doc = "Delete subnet."]
        #[derive(clap::Parser, Debug, Clone)]
        #[clap(verbatim_doc_comment)]
        pub struct CmdSubnetDelete {
            #[doc = "The subnet to delete. Can be an ID or name."]
            #[clap(name = "subnet", required = true)]
            pub subnet: String,

            #[doc = "The project to delete the subnet from."]
            #[clap(long, short, required = true)]
            pub project: String,

            /// The organization that holds the project.
            #[clap(long, short, required = true, env = "OXIDE_ORG")]
            pub organization: String,

            #[doc = "The VPC that holds the subnet."]
            #[clap(long, short, required = true)]
            pub vpc: String,

            /// Confirm deletion without prompting.
            #[clap(long)]
            pub confirm: bool,
        }

        #[async_trait::async_trait]
        impl crate::cmd::Command for CmdSubnetDelete {
            async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                if !ctx.io.can_prompt() && !self.confirm {
                    return Err(anyhow::anyhow!("--confirm required when not running interactively"));
                }

                let client = ctx.api_client("")?;

                // Confirm deletion.
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

                // Delete the project.
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
    };

    assert_eq!(expected.to_string(), ret.unwrap().to_string());
}
