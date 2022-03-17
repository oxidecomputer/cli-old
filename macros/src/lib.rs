extern crate proc_macro;

use anyhow::Result;
use inflector::cases::titlecase::to_title_case;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use syn::{Field, ItemEnum};

/// The parameters passed to our macro.
#[derive(Deserialize, Debug)]
struct Params {
    /// The name of the tag that the commands are grouped buy.
    tag: String,
}

#[proc_macro_attribute]
pub fn crud_gen(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_gen(attr.into(), item.into()).unwrap().into()
}

fn do_gen(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Get the data from the parameters.
    let params = from_tokenstream::<Params>(&attr)?;

    // Lets get the Open API spec.
    let api: openapiv3::OpenAPI = load_api_spec()?;

    let ops = get_operations_with_tag(&api, &params.tag)?;

    let og_enum: ItemEnum = syn::parse2(item.clone()).unwrap();
    let mut variants = og_enum.variants.clone();
    let mut commands = quote!();

    // Let's iterate over the paths and generate the code.
    for op in ops {
        // Let's generate the delete command if it exists.
        if op.is_root_level_operation(&params.tag) && op.method == "DELETE" {
            let (delete_cmd, delete_enum_item) = op.generate_delete_command(&params.tag)?;

            commands = quote! {
                #commands

                #delete_cmd
            };

            variants.push(delete_enum_item);
        }
    }

    let attrs = og_enum.attrs.clone();
    let code = quote!(
        #(#attrs);*
        enum SubCommand {
            #variants
        }

        #commands
    );

    Ok(code)
}

/// Get the OpenAPI spec from the file.
fn load_api_spec<T>() -> Result<T>
where
    for<'de> T: Deserialize<'de>,
{
    let s = include_str!("../../spec.json");
    Ok(serde_json::from_str(s)?)
}

trait ReferenceOrExt<T> {
    fn item(&self) -> Result<&T>;
}

impl<T> ReferenceOrExt<T> for openapiv3::ReferenceOr<T> {
    fn item(&self) -> Result<&T> {
        match self {
            openapiv3::ReferenceOr::Item(i) => Ok(i),
            openapiv3::ReferenceOr::Reference { reference } => {
                anyhow::bail!("reference not supported here: {}", reference);
            }
        }
    }
}

struct Operation {
    op: openapiv3::Operation,
    method: String,
    path: String,
    id: String,
}

impl Operation {
    /// Returns if the given operation is a root level operation on a specific tag.
    fn is_root_level_operation(&self, tag: &str) -> bool {
        self.id
            .ends_with(&format!("{}_{}", self.method.to_lowercase(), singular(tag)))
    }

    /// Generate the delete command.
    fn generate_delete_command(&self, tag: &str) -> Result<(TokenStream, syn::Variant)> {
        let tag_ident = format_ident!("{}", tag);
        let singular_tag_str = singular(tag);
        let singular_tag_lc = format_ident!("{}", singular_tag_str);
        let struct_name = format_ident!("Cmd{}Delete", to_title_case(&singular_tag_str));

        let struct_doc = format!("Delete a {}.", singular(tag));
        let struct_inner_name_doc = format!("The {} to delete. Can be an ID or name.", singular_tag_str);
        let struct_inner_project_doc = format!("The project to delete the {} from.", singular_tag_str);

        let cmd = quote!(
            #[doc = #struct_doc]
            #[derive(clap::Parser, Debug, Clone)]
            #[clap(verbatim_doc_comment)]
            pub struct #struct_name {
                #[doc = #struct_inner_name_doc]
                #[clap(name = #singular_tag_str, required = true)]
                #singular_tag_lc: String,

                #[doc = #struct_inner_project_doc]
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
            impl crate::cmd::Command for #struct_name {
                async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                    if !ctx.io.can_prompt() && !self.confirm {
                        return Err(anyhow!("--confirm required when not running interactively"));
                    }

                    let client = ctx.api_client("")?;

                    let full_name = format!("{}/{}", self.organization, self.project);

                    // Confirm deletion.
                    if !self.confirm {
                        if let Err(err) = dialoguer::Input::<String>::new()
                            .with_prompt(format!("Type {} to confirm deletion:", self.#singular_tag_lc))
                            .validate_with(|input: &String| -> Result<(), &str> {
                                if input.trim() == full_name {
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
                        .#tag_ident()
                        .delete(&self.#singular_tag_lc, &self.organization, &self.project)
                        .await?;

                    let cs = ctx.io.color_scheme();
                    writeln!(
                        ctx.io.out,
                        "{} Deleted {} {} from {}",
                        cs.success_icon_with_color(ansi_term::Color::Red),
                        #singular_tag_str,
                        self.#singular_tag_lc,
                        full_name
                    )?;

                    Ok(())
                }
            }
        );

        let enum_item: syn::Variant = syn::parse2(quote!(Delete(#struct_name)))?;

        Ok((cmd, enum_item))
    }
}

/// Get the operations with the tag from the OpenAPI spec.
fn get_operations_with_tag(api: &openapiv3::OpenAPI, tag: &str) -> Result<Vec<Operation>> {
    let mut paths = Vec::new();

    for (pn, p) in api.paths.iter() {
        let op = p.item()?;

        let grab = |pn: &str, m: &str, o: Option<&openapiv3::Operation>| -> Result<Vec<Operation>> {
            if let Some(o) = o {
                for t in o.tags.iter() {
                    if t == tag {
                        let id = if let Some(i) = o.operation_id.as_ref() {
                            i.to_string()
                        } else {
                            "".to_string()
                        };

                        return Ok(vec![Operation {
                            op: o.clone(),
                            method: m.to_string(),
                            path: pn.to_string(),
                            id,
                        }]);
                    }
                }
            }

            Ok(Default::default())
        };

        paths.append(&mut grab(pn, "GET", op.get.as_ref())?);
        paths.append(&mut grab(pn, "POST", op.post.as_ref())?);
        paths.append(&mut grab(pn, "PUT", op.put.as_ref())?);
        paths.append(&mut grab(pn, "DELETE", op.delete.as_ref())?);
        paths.append(&mut grab(pn, "OPTIONS", op.options.as_ref())?);
        paths.append(&mut grab(pn, "HEAD", op.head.as_ref())?);
        paths.append(&mut grab(pn, "PATCH", op.patch.as_ref())?);
        paths.append(&mut grab(pn, "TRACE", op.trace.as_ref())?);
    }

    Ok(paths)
}

/// Return the singular version of a string (if it plural).
fn singular(s: &str) -> String {
    if let Some(b) = s.strip_suffix('s') {
        b.to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_crud_gen() {
        let ret = do_gen(
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
                    List(CmdDiskList),
                    View(CmdDiskView),
                }
            },
        );
        let expected = quote! {
            #[derive(Parser, Debug, Clone)]
            enum SubCommand {
                Attach(CmdDiskAttach),
                Create(CmdDiskCreate),
                Detach(CmdDiskDetach),
                Edit(CmdDiskEdit),
                List(CmdDiskList),
                View(CmdDiskView),
                Delete(CmdDiskDelete)
            }

            #[doc = "Delete a disk."]
            #[derive(clap::Parser, Debug, Clone)]
            #[clap(verbatim_doc_comment)]
            pub struct CmdDiskDelete {
                #[doc = "The disk to delete. Can be an ID or name."]
                #[clap(name = "disk", required = true)]
                disk: String,

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
                        return Err(anyhow!("--confirm required when not running interactively"));
                    }

                    let client = ctx.api_client("")?;

                    let full_name = format!("{}/{}", self.organization, self.project);

                    // Confirm deletion.
                    if !self.confirm {
                        if let Err(err) = dialoguer::Input::<String>::new()
                            .with_prompt(format!("Type {} to confirm deletion:", self.disk))
                            .validate_with(|input: &String| -> Result<(), &str> {
                                if input.trim() == full_name {
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
                        .disks()
                        .delete(&self.disk, &self.organization, &self.project)
                        .await?;

                    let cs = ctx.io.color_scheme();
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
    }
}
