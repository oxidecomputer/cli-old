extern crate proc_macro;

use std::collections::BTreeMap;

use anyhow::Result;
use inflector::cases::titlecase::to_title_case;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_tokenstream::from_tokenstream;
use syn::ItemEnum;

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

    let og_enum: ItemEnum = syn::parse2(item).unwrap();
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

            // Clap with alphabetize the help text subcommands so it is fine to just shove
            // the variants on the end.
            variants.push(delete_enum_item);
        }
    }

    let attrs = og_enum.attrs;
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

trait ParameterExt {
    fn data(&self) -> Option<openapiv3::ParameterData>;
}

impl ParameterExt for openapiv3::Parameter {
    fn data(&self) -> Option<openapiv3::ParameterData> {
        match self {
            openapiv3::Parameter::Path {
                parameter_data,
                style: openapiv3::PathStyle::Simple,
            } => return Some(parameter_data.clone()),
            openapiv3::Parameter::Header {
                parameter_data,
                style: openapiv3::HeaderStyle::Simple,
            } => return Some(parameter_data.clone()),
            openapiv3::Parameter::Cookie {
                parameter_data,
                style: openapiv3::CookieStyle::Form,
            } => return Some(parameter_data.clone()),
            openapiv3::Parameter::Query {
                parameter_data,
                allow_reserved: _,
                style: openapiv3::QueryStyle::Form,
                allow_empty_value: _,
            } => {
                return Some(parameter_data.clone());
            }
            _ => (),
        }

        None
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

    fn get_parameters(&self) -> Result<BTreeMap<String, openapiv3::Parameter>> {
        let mut parameters = BTreeMap::new();

        for param in self.op.parameters.iter() {
            let param = param.item()?;

            let parameter_data = match param.data() {
                Some(s) => s,
                None => return Ok(parameters),
            };

            parameters.insert(parameter_data.name.to_string(), param.clone());
        }

        Ok(parameters)
    }

    fn is_parameter(&self, parameter: &str) -> bool {
        for param in self.op.parameters.iter() {
            let param = match param.item() {
                Ok(i) => i,
                Err(_) => return false,
            };

            let parameter_data = match param.data() {
                Some(s) => s,
                None => return false,
            };

            if parameter_data.name == parameter || parameter_data.name.starts_with(&format!("{}_", parameter)) {
                return true;
            }
        }

        false
    }

    fn get_request_body_properties(&self) -> Result<BTreeMap<String, Box<openapiv3::Schema>>> {
        let mut properties = BTreeMap::new();

        let request_body = match self.op.request_body.as_ref() {
            Some(r) => r,
            None => return Ok(properties),
        }
        .item()?;

        let content = match request_body.content.get("application/json") {
            Some(c) => c,
            None => return Ok(properties),
        };

        let schema = match content.schema.as_ref() {
            Some(s) => s,
            None => return Ok(properties),
        }
        .item()?;

        let obj = match &schema.schema_kind {
            openapiv3::SchemaKind::Type(t) => match t {
                openapiv3::Type::Object(o) => o,
                _ => return Ok(properties),
            },
            _ => return Ok(properties),
        };

        for (key, prop) in obj.properties.iter() {
            properties.insert(key.clone(), prop.item()?.clone());
        }

        Ok(properties)
    }

    fn is_request_body_property(&self, property: &str) -> bool {
        let request_body = match self.op.request_body.as_ref() {
            Some(r) => r,
            None => return false,
        };

        let request_body = match request_body.item() {
            Ok(i) => i,
            Err(_) => return false,
        };

        let content = match request_body.content.get("application/json") {
            Some(c) => c,
            None => return false,
        };

        let schema = match content.schema.as_ref() {
            Some(s) => s,
            None => return false,
        };

        let schema = match schema.item() {
            Ok(s) => s,
            Err(_) => return false,
        };

        let obj = match &schema.schema_kind {
            openapiv3::SchemaKind::Type(t) => match t {
                openapiv3::Type::Object(o) => o,
                _ => return false,
            },
            _ => return false,
        };

        for (key, _) in obj.properties.iter() {
            if key == property {
                return true;
            }
        }

        false
    }

    /// Gets a list of all the string parameters for the operation.
    /// This includes the path parameters, query parameters, and request_body parameters.
    fn get_all_param_names(&self) -> Result<Vec<String>> {
        let mut param_names = Vec::new();

        for param in self.get_parameters()?.keys() {
            param_names.push(param.to_string());
        }

        for param in self.get_request_body_properties()?.keys() {
            param_names.push(param.to_string());
        }

        // Since we sort in the client, we also sort here such that parameters wind up in the right
        // order always.
        param_names.sort();

        Ok(param_names)
    }

    /// Generate the delete command.
    fn generate_delete_command(&self, tag: &str) -> Result<(TokenStream, syn::Variant)> {
        let tag_ident = format_ident!("{}", tag);
        let singular_tag_str = if tag == "vpcs" {
            singular(tag).to_uppercase()
        } else {
            singular(tag)
        };
        let singular_tag_lc = format_ident!("{}", singular(tag));
        let struct_name = format_ident!("Cmd{}Delete", to_title_case(&singular(tag)));

        let struct_doc = format!("Delete a {}.", singular_tag_str);
        let struct_inner_name_doc = format!("The {} to delete. Can be an ID or name.", singular_tag_str);
        let struct_inner_project_doc = format!("The project to delete the {} from.", singular_tag_str);

        let mut api_call_params: Vec<TokenStream> = Vec::new();
        for p in self.get_all_param_names()? {
            let p = format_ident!("{}", p.trim_end_matches("_name"));
            api_call_params.push(quote!(&self.#p));
        }

        // We need to check if project is a parameter to this call.
        let project_param = if self.is_parameter("project") && tag != "projects" {
            quote! {
                #[doc = #struct_inner_project_doc]
                #[clap(long, short, required = true)]
                pub project: String,
            }
        } else {
            quote!()
        };

        // We need to check if organization is a parameter to this call.
        let organization_param = if self.is_parameter("organization") && tag != "organizations" {
            quote! {
                /// The organization that holds the project.
                #[clap(long, short, required = true, env = "OXIDE_ORG")]
                pub organization: String,
            }
        } else {
            quote!()
        };

        // We need to form the output back to the client.
        let output = if self.is_parameter("organization") && self.is_parameter("project") {
            let start = quote! {
                let full_name = format!("{}/{}", self.organization, self.project);
            };
            if tag != "projects" {
                quote! {
                    #start
                    writeln!(
                        ctx.io.out,
                        "{} Deleted {} {} from {}",
                        cs.success_icon_with_color(ansi_term::Color::Red),
                        #singular_tag_str,
                        self.#singular_tag_lc,
                        full_name
                    )?;
                }
            } else {
                quote! {
                    #start
                    writeln!(
                        ctx.io.out,
                        "{} Deleted {} {}",
                        cs.success_icon_with_color(ansi_term::Color::Red),
                        #singular_tag_str,
                        full_name
                    )?;
                }
            }
        } else {
            quote! {
                writeln!(
                    ctx.io.out,
                    "{} Deleted {} {}",
                    cs.success_icon_with_color(ansi_term::Color::Red),
                    #singular_tag_str,
                    self.#singular_tag_lc
                )?;
            }
        };

        let cmd = quote!(
            #[doc = #struct_doc]
            #[derive(clap::Parser, Debug, Clone)]
            #[clap(verbatim_doc_comment)]
            pub struct #struct_name {
                #[doc = #struct_inner_name_doc]
                #[clap(name = #singular_tag_str, required = true)]
                #singular_tag_lc: String,

                #project_param

                #organization_param

                /// Confirm deletion without prompting.
                #[clap(long)]
                pub confirm: bool,
            }

            #[async_trait::async_trait]
            impl crate::cmd::Command for #struct_name {
                async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                    if !ctx.io.can_prompt() && !self.confirm {
                        return Err(anyhow::anyhow!("--confirm required when not running interactively"));
                    }

                    let client = ctx.api_client("")?;


                    // Confirm deletion.
                    if !self.confirm {
                        if let Err(err) = dialoguer::Input::<String>::new()
                            .with_prompt(format!("Type {} to confirm deletion:", self.#singular_tag_lc))
                            .validate_with(|input: &String| -> Result<(), &str> {
                                if input.trim() == self.#singular_tag_lc {
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
                        .#tag_ident()
                        .delete(#(#api_call_params),*,)
                        .await?;

                    let cs = ctx.io.color_scheme();

                    #output

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
                    List(CmdDiskList),
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
                        .delete(&self.disk, &self.organization, &self.project,)
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
                Delete(CmdOrganizationDelete)
            }

            #[doc = "Delete a organization."]
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

                    client.organizations().delete(&self.organization,).await?;

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
    }
}
