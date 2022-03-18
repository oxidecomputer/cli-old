#[cfg(test)]
mod tests;

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
        } else if op.is_root_list_operation(&params.tag) {
            let (list_cmd, list_enum_item) = op.generate_list_command(&params.tag)?;

            commands = quote! {
                #commands

                #list_cmd
            };

            // Clap with alphabetize the help text subcommands so it is fine to just shove
            // the variants on the end.
            variants.push(list_enum_item);
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
    fn reference(&self) -> Result<String>;
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

    fn reference(&self) -> Result<String> {
        match self {
            openapiv3::ReferenceOr::Item(..) => {
                anyhow::bail!("item not supported here");
            }
            openapiv3::ReferenceOr::Reference { reference } => {
                Ok(reference.trim_start_matches("#/components/schemas/").to_string())
            }
        }
    }
}

trait ParameterSchemaOrContentExt {
    fn schema(&self) -> Result<openapiv3::ReferenceOr<openapiv3::Schema>>;
}

impl ParameterSchemaOrContentExt for openapiv3::ParameterSchemaOrContent {
    fn schema(&self) -> Result<openapiv3::ReferenceOr<openapiv3::Schema>> {
        match self {
            openapiv3::ParameterSchemaOrContent::Schema(s) => Ok(s.clone()),
            openapiv3::ParameterSchemaOrContent::Content(..) => {
                anyhow::bail!("content not supported here");
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
    #[allow(dead_code)]
    path: String,
    id: String,
}

impl Operation {
    /// Returns if the given operation is a root level operation on a specific tag.
    fn is_root_level_operation(&self, tag: &str) -> bool {
        self.id
            .ends_with(&format!("{}_{}", self.method.to_lowercase(), singular(tag)))
    }

    /// Returns if the given operation is a root list operation on a specific tag.
    fn is_root_list_operation(&self, tag: &str) -> bool {
        let pagination =
            if let Some(serde_json::value::Value::Bool(b)) = self.op.extensions.get("x-dropshot-pagination") {
                *b
            } else {
                return false;
            };

        self.id.ends_with(&format!("{}_{}", tag, self.method.to_lowercase())) && pagination && self.method == "GET"
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
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(o)) => o,
            _ => return Ok(properties),
        };

        for (key, prop) in obj.properties.iter() {
            properties.insert(key.clone(), prop.item()?.clone());
        }

        Ok(properties)
    }

    #[allow(dead_code)]
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
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(o)) => o,
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

        param_names.sort();

        Ok(param_names)
    }

    /// Get additional struct parameters.
    fn get_additional_struct_params(&self, tag: &str) -> Result<Vec<TokenStream>> {
        let mut params = Vec::new();

        for (param, p) in self.get_parameters()? {
            if param == "project"
                || param == "organization"
                || param == "project_name"
                || param == "organization_name"
                || param == singular(tag)
                || param == format!("{}_name", singular(tag))
                || param == "limit"
                || param == "page_token"
            {
                continue;
            }

            let data = if let Some(data) = p.data() {
                data
            } else {
                continue;
            };

            let name = param.trim_end_matches("_name");
            let p_ident = format_ident!("{}", name);
            let param_doc = if let Some(desc) = &data.description {
                desc.to_string()
            } else if name == "sort_by" {
                "The order in which to sort the results.".to_string()
            } else {
                let n = if name == "vpc" {
                    name.to_uppercase()
                } else {
                    name.to_string()
                };

                if self.is_root_list_operation(tag) {
                    format!("The {} that holds the {}.", n, plural(tag))
                } else {
                    format!("The {} that holds the {}.", n, singular(tag))
                }
            };

            if name == "sort_by" {
                let type_ident = format_ident!("{}", data.format.schema()?.reference()?);
                // TODO: set the default sort mode.
                params.push(quote! {
                    #[doc = #param_doc]
                    #[clap(long, short)]
                    pub #p_ident: oxide_api::types::#type_ident,
                });
            } else {
                params.push(quote! {
                    #[doc = #param_doc]
                    #[clap(long, short, required = true)]
                    pub #p_ident: String,
                });
            }
        }

        Ok(params)
    }

    /// Generate the list command.
    fn generate_list_command(&self, tag: &str) -> Result<(TokenStream, syn::Variant)> {
        let tag_ident = format_ident!("{}", tag);
        let singular_tag_str = if tag == "vpcs" {
            singular(tag).to_uppercase()
        } else {
            singular(tag)
        };
        let struct_name = format_ident!("Cmd{}List", to_title_case(&singular(tag)));

        let struct_doc = format!("List {}.", plural(&singular_tag_str));
        let struct_inner_project_doc = format!("The project that holds the {}.", plural(&singular_tag_str));

        let mut api_call_params_all: Vec<TokenStream> = Vec::new();
        let mut api_call_params: Vec<TokenStream> = Vec::new();
        for p in self.get_all_param_names()? {
            // TODO: we should support sort by.
            if p == "page_token" {
                api_call_params.push(quote!(""));
                continue;
            }

            if p == "limit" {
                api_call_params.push(quote!(self.limit));
                continue;
            }

            let p = format_ident!("{}", p.trim_end_matches("_name"));

            if p == "sort_by" {
                // Sort by is an enum so we don't want to "&" it
                api_call_params_all.push(quote!(self.#p.clone()));
                api_call_params.push(quote!(self.#p.clone()));
                continue;
            }

            api_call_params_all.push(quote!(&self.#p));
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

        let additional_struct_params = self.get_additional_struct_params(tag)?;

        let cmd = quote!(
            #[doc = #struct_doc]
            #[derive(clap::Parser, Debug, Clone)]
            #[clap(verbatim_doc_comment)]
            pub struct #struct_name {
                #project_param

                #organization_param

                #(#additional_struct_params)*

                /// Maximum number of items to list.
                #[clap(long, short, default_value = "30")]
                pub limit: u32,

                /// Make additional HTTP requests to fetch all pages.
                #[clap(long)]
                pub paginate: bool,

                // TODO: Change this to be format instead!
                /// Output JSON.
                #[clap(long)]
                pub json: bool,
            }

            #[async_trait::async_trait]
            impl crate::cmd::Command for #struct_name {
                async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                    if self.limit < 1 {
                    return Err(anyhow::anyhow!("--limit must be greater than 0"));
                }

                let client = ctx.api_client("")?;

                let results = if self.paginate {
                    client
                        .#tag_ident()
                        .get_all(
                            #(#api_call_params_all),*
                        )
                        .await?
                } else {
                    client
                        .#tag_ident()
                        .get_page(
                            #(#api_call_params),*
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
        );

        let enum_item: syn::Variant = syn::parse2(quote!(List(#struct_name)))?;

        Ok((cmd, enum_item))
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

        let struct_doc = format!("Delete {}.", singular_tag_str);
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

        let additional_struct_params = self.get_additional_struct_params(tag)?;

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
                pub #singular_tag_lc: String,

                #project_param

                #organization_param

                #(#additional_struct_params)*

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
                        .delete(#(#api_call_params),*)
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

/// Return the plural version of a string.
fn plural(s: &str) -> String {
    let s = singular(s);

    if s.ends_with('s') {
        return format!("{}es", s);
    } else if s.ends_with('y') {
        return format!("{}ies", s.trim_end_matches('y'));
    }

    format!("{}s", s)
}

/// Return the singular version of a string (if it plural).
fn singular(s: &str) -> String {
    if let Some(b) = s.strip_suffix('s') {
        return b.to_string();
    }

    s.to_string()
}
