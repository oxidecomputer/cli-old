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

pub fn do_gen(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Get the data from the parameters.
    let params = from_tokenstream::<Params>(&attr)?;

    // Lets get the Open API spec.
    let api = load_api_spec()?;

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
        } else if op.is_root_level_operation(&params.tag) && op.method == "GET" {
            let (view_cmd, view_enum_item) = op.generate_view_command(&params.tag)?;

            commands = quote! {
                #commands

                #view_cmd
            };

            // Clap with alphabetize the help text subcommands so it is fine to just shove
            // the variants on the end.
            variants.push(view_enum_item);
        } else if op.is_root_create_operation(&params.tag) {
            let (create_cmd, create_enum_item) = op.generate_create_command(&params.tag)?;

            commands = quote! {
                #commands

                #create_cmd
            };

            // Clap with alphabetize the help text subcommands so it is fine to just shove
            // the variants on the end.
            variants.push(create_enum_item);
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
fn load_api_spec() -> Result<openapiv3::OpenAPI> {
    let s = include_str!("../../spec.json");
    Ok(serde_json::from_str(s)?)
}

trait ReferenceOrExt<T> {
    fn item(&self) -> Result<&T>;
    fn reference(&self) -> Result<String>;
    fn reference_render_type(&self) -> Result<TokenStream>;
    fn get_schema_from_reference(&self) -> Result<openapiv3::Schema>;
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

    fn reference_render_type(&self) -> Result<TokenStream> {
        let name = self.reference()?;
        let ident = format_ident!("{}", name);

        // We want the full path to the type.
        Ok(quote!(oxide_api::types::#ident))
    }

    fn get_schema_from_reference(&self) -> Result<openapiv3::Schema> {
        let name = self.reference()?;

        let spec = load_api_spec()?;

        let components = spec
            .components
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("components not found in spec"))?;

        let schema = components
            .schemas
            .get(&name)
            .ok_or_else(|| anyhow::anyhow!("could not find schema with name {}", name))?;

        Ok(schema.item()?.clone())
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

trait SchemaExt {
    fn render_type(&self) -> Result<TokenStream>;
}

impl SchemaExt for openapiv3::Schema {
    fn render_type(&self) -> Result<TokenStream> {
        match &self.schema_kind {
            openapiv3::SchemaKind::Type(openapiv3::Type::Boolean {}) => Ok(quote!(bool)),
            openapiv3::SchemaKind::Type(openapiv3::Type::Array(a)) => {
                let schema = a
                    .items
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("no items in array `{:#?}`", a))?;
                match schema.item() {
                    Ok(s) => s.render_type(),
                    Err(e) => {
                        if e.to_string().contains("reference") {
                            schema.reference_render_type()
                        } else {
                            anyhow::bail!("could not get schema type from array `{:#?}`: {}", a, e);
                        }
                    }
                }
            }
            openapiv3::SchemaKind::Type(openapiv3::Type::String(st)) => {
                if !st.enumeration.is_empty() {
                    anyhow::bail!("enumeration not supported here yet: {:?}", st);
                }

                Ok(match &st.format {
                    openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::DateTime) => {
                        quote!(chrono::DateTime<chrono::Utc>)
                    }
                    openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Date) => {
                        quote!(chrono::NaiveDate)
                    }
                    openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Password) => quote!(String),
                    // TODO: as per the spec this is base64 encoded chars.
                    openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Byte) => {
                        quote!(bytes::Bytes)
                    }
                    openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::StringFormat::Binary) => {
                        quote!(bytes::Bytes)
                    }
                    openapiv3::VariantOrUnknownOrEmpty::Empty => quote!(String),
                    openapiv3::VariantOrUnknownOrEmpty::Unknown(f) => match f.as_str() {
                        "float" => quote!(f64),
                        "int64" => quote!(i64),
                        "uint64" => quote!(u64),
                        "ipv4" => quote!(std::net::Ipv4Addr),
                        "ip" => quote!(std::net::Ipv4Addr),
                        "uri" => quote!(url::Url),
                        "uri-template" => quote!(String),
                        "url" => quote!(url::Url),
                        "email" => quote!(String),
                        "uuid" => quote!(uuid::Uuid),
                        "hostname" => quote!(String),
                        "time" => quote!(chrono::NaiveTime),
                        f => {
                            anyhow::bail!("XXX unknown string format {}", f)
                        }
                    },
                })
            }
            openapiv3::SchemaKind::Type(openapiv3::Type::Number(_)) => Ok(quote!(f64)),
            openapiv3::SchemaKind::Type(openapiv3::Type::Integer(it)) => {
                let uint;
                let width;

                if let openapiv3::VariantOrUnknownOrEmpty::Unknown(f) = &it.format {
                    match f.as_str() {
                        "uint" | "uint32" => {
                            uint = true;
                            width = 32;
                        }
                        "uint8" => {
                            uint = true;
                            width = 8;
                        }
                        "uint16" => {
                            uint = true;
                            width = 16;
                        }
                        "uint64" => {
                            uint = true;
                            width = 64;
                        }
                        f => anyhow::bail!("unknown integer format {}", f),
                    }
                } else {
                    // The format was empty, let's assume it's just a normal
                    // i64.
                    uint = false;
                    width = 64;
                }

                Ok(if uint {
                    match width {
                        8 => quote!(u8),
                        16 => quote!(u16),
                        32 => quote!(u32),
                        64 => quote!(u64),
                        _ => anyhow::bail!("unknown uint width {}", width),
                    }
                } else {
                    match width {
                        8 => quote!(i8),
                        16 => quote!(i16),
                        32 => quote!(i32),
                        64 => quote!(i64),
                        _ => anyhow::bail!("unknown int width {}", width),
                    }
                })
            }
            openapiv3::SchemaKind::OneOf { one_of: _ } => {
                // TODO: Turn this on.
                Ok(quote!(String))
                //anyhow::bail!("oneOf not supported here yet: {:?}", one_of)
            }
            openapiv3::SchemaKind::Any(any) => {
                anyhow::bail!("any not supported here yet: {:?}", any)
            }
            openapiv3::SchemaKind::AllOf { all_of } => {
                if all_of.len() != 1 {
                    anyhow::bail!(
                        "allOf length is `{}`, only len == 1 supported: {:#?}",
                        all_of.len(),
                        all_of
                    )
                }

                let schema = all_of.get(0).unwrap();
                match schema.item() {
                    Ok(s) => s.render_type(),
                    Err(e) => {
                        if e.to_string().contains("reference") {
                            schema.reference_render_type()
                        } else {
                            anyhow::bail!("could not get schema type from allOf `{:#?}`: {}", all_of, e);
                        }
                    }
                }
            }
            x => anyhow::bail!("unexpected type {:#?}", x),
        }
    }
}

struct Operation {
    op: openapiv3::Operation,
    method: String,
    #[allow(dead_code)]
    path: String,
    id: String,
}

struct Property {
    schema: openapiv3::Schema,
    required: bool,
}

struct Parameter {
    parameter: openapiv3::Parameter,
    required: bool,
}

impl Parameter {
    fn data(&self) -> Option<openapiv3::ParameterData> {
        self.parameter.data()
    }
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

    /// Returns if the given operation is a root create operation on a specific tag.
    fn is_root_create_operation(&self, tag: &str) -> bool {
        self.id.ends_with(&format!("{}_{}", tag, self.method.to_lowercase())) && self.method == "POST"
    }

    fn get_parameters(&self) -> Result<BTreeMap<String, Parameter>> {
        let mut parameters = BTreeMap::new();

        for param in self.op.parameters.iter() {
            let param = param.item()?;

            let parameter_data = match param.data() {
                Some(s) => s,
                None => return Ok(parameters),
            };

            parameters.insert(
                parameter_data.name.to_string(),
                Parameter {
                    parameter: param.clone(),
                    required: parameter_data.required,
                },
            );
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

    fn get_request_body_name(&self) -> Result<String> {
        let request_body = match self.op.request_body.as_ref() {
            Some(r) => r,
            None => anyhow::bail!("no request_body found"),
        }
        .item()?;

        let content = match request_body.content.get("application/json") {
            Some(c) => c,
            None => anyhow::bail!("no `application/json` found"),
        };

        let schema = match content.schema.as_ref() {
            Some(s) => s,
            None => anyhow::bail!("no content schema found"),
        };

        schema.reference()
    }

    fn get_request_body_properties(&self) -> Result<BTreeMap<String, Property>> {
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
        };

        let schema = match schema.item() {
            Ok(b) => b.clone(),
            Err(e) => {
                if e.to_string().contains("reference") {
                    schema.get_schema_from_reference()?
                } else {
                    anyhow::bail!("could not get schema from request body: {}", e);
                }
            }
        };

        let obj = match &schema.schema_kind {
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(o)) => o,
            _ => return Ok(properties),
        };

        for (key, prop) in obj.properties.iter() {
            let s = match prop.item() {
                Ok(s) => *s.clone(),
                Err(e) => {
                    if e.to_string().contains("reference") {
                        prop.get_schema_from_reference()?
                    } else {
                        anyhow::bail!("could not get schema from prop `{}`: {}", key, e);
                    }
                }
            };
            properties.insert(
                key.clone(),
                Property {
                    schema: s,
                    required: obj.required.contains(key),
                },
            );
        }

        Ok(properties)
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

    /// Gets all the api call params for the operation.
    /// This includes the path parameters, query parameters, and request_body parameters.
    fn get_api_call_params(&self, tag: &str) -> Result<Vec<TokenStream>> {
        let mut api_call_params: Vec<TokenStream> = Vec::new();

        let params = self.get_parameters()?;
        let mut params = params.keys().collect::<Vec<_>>();
        params.sort();

        for p in params {
            if p == "page_token" {
                api_call_params.push(quote!(""));
                continue;
            }

            if p == "limit" {
                api_call_params.push(quote!(self.limit));
                continue;
            }

            let p = format_ident!("{}", p.trim_end_matches("_name").trim_end_matches("_id"));

            if p == "sort_by" {
                // Sort by is an enum so we don't want to "&" it
                api_call_params.push(quote!(self.#p.clone()));
                continue;
            }

            api_call_params.push(quote!(&self.#p));
        }

        let req_body_properties = self.get_request_body_properties()?;
        let req_body_properties = req_body_properties.keys();
        if req_body_properties.len() > 0 {
            let req_body_properties = req_body_properties
                .map(|p| {
                    let p_og = format_ident!("{}", p);

                    let new = if p == "name" { singular(tag) } else { p.to_string() };
                    let p_short = format_ident!("{}", new.trim_end_matches("_name").trim_end_matches("_id"));

                    quote!(#p_og: #p_short.clone())
                })
                .collect::<Vec<_>>();

            let type_name = self.get_request_body_name()?;
            let type_name = format_ident!("{}", type_name);

            api_call_params.push(quote! {
                &oxide_api::types::#type_name {
                    #(#req_body_properties),*
                }
            });
        }

        Ok(api_call_params)
    }

    /// Gets a list of all the required string parameters for the operation.
    /// This includes the path parameters, query parameters, and request_body parameters.
    fn get_all_required_param_names(&self) -> Result<Vec<String>> {
        let mut param_names = Vec::new();

        for (param, p) in self.get_parameters()? {
            if p.data().unwrap().required {
                param_names.push(param.to_string());
            }
        }

        for (param, p) in self.get_request_body_properties()? {
            if p.required {
                param_names.push(param.to_string());
            }
        }

        param_names.sort();

        Ok(param_names)
    }

    fn render_struct_param<T: SchemaExt>(
        &self,
        name: &str,
        tag: &str,
        schema: openapiv3::ReferenceOr<T>,
        description: Option<String>,
        required: bool,
    ) -> Result<TokenStream> {
        if name == "project"
            || name == "name"
            || name == "organization"
            || name == "project_name"
            || name == "organization_name"
            || name == singular(tag)
            || name == format!("{}_name", singular(tag))
            || name == format!("{}_id", singular(tag))
            || name == "limit"
            || name == "page_token"
        {
            // Return early and empty, we don't care about these.
            return Ok(quote!());
        }

        let name_cleaned = name.trim_end_matches("_name").trim_end_matches("_id");
        let name_ident = format_ident!("{}", name_cleaned);

        let doc = if let Some(desc) = description {
            desc.to_string()
        } else if name == "sort_by" {
            "The order in which to sort the results.".to_string()
        } else {
            let n = if name_cleaned == "vpc" {
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

        let type_name = match schema.item() {
            Ok(s) => s.render_type()?,
            Err(e) => {
                if e.to_string().contains("reference") {
                    schema.reference_render_type()?
                } else {
                    anyhow::bail!("could not get schema type from param `{}`: {}", name, e);
                }
            }
        };

        let clap_line = if self.method == "POST" || name == "sort_by" {
            // On create, we want to set default values for the parameters.
            quote! {
                #[clap(long, short, default_value_t)]
            }
        } else {
            let required = if required { quote!(true) } else { quote!(false) };

            quote! {
                #[clap(long, short, required = #required)]
            }
        };

        Ok(quote! {
            #[doc = #doc]
            #clap_line
            pub #name_ident: #type_name,
        })
    }

    /// Get additional struct parameters.
    fn get_additional_struct_params(&self, tag: &str) -> Result<Vec<TokenStream>> {
        let mut params = Vec::new();

        for (param, p) in self.get_parameters()? {
            let data = if let Some(data) = p.data() {
                data
            } else {
                continue;
            };

            // Let's get the type.
            let schema = data.format.schema()?;

            params.push(self.render_struct_param(&param, tag, schema, data.description, p.required)?);
        }

        for (param, p) in self.get_request_body_properties()? {
            params.push(self.render_struct_param(
                &param,
                tag,
                openapiv3::ReferenceOr::Item(p.schema.clone()),
                p.schema.schema_data.description,
                p.required,
            )?);
        }

        Ok(params)
    }

    /// Generate the create command.
    fn generate_create_command(&self, tag: &str) -> Result<(TokenStream, syn::Variant)> {
        let tag_ident = format_ident!("{}", tag);
        let singular_tag_str = if tag == "vpcs" {
            singular(tag).to_uppercase()
        } else {
            singular(tag)
        };
        let singular_tag_lc = format_ident!("{}", singular(tag));
        let struct_name = format_ident!("Cmd{}Create", to_title_case(&singular(tag)));

        let struct_doc = format!(
            "Create a new {}.\n\nTo create a {} interactively, use `oxide {} create` with no arguments.",
            singular_tag_str,
            singular_tag_str,
            &singular(tag)
        );
        let struct_inner_project_doc = format!("The project that holds the {}.", singular_tag_str);

        let struct_inner_name_doc = format!("The name of the {} to create.", singular_tag_str);

        let mut mutable_variables: Vec<TokenStream> = Vec::new();
        for p in self.get_all_param_names()? {
            let p = if p == "name" { singular(tag) } else { p };
            let p = format_ident!("{}", p.trim_end_matches("_name").trim_end_matches("_id"));

            mutable_variables.push(quote!(
                let mut #p = self.#p.clone();
            ));
        }

        let api_call_params = self.get_api_call_params(tag)?;

        let mut required_checks: Vec<TokenStream> = Vec::new();
        for p in self.get_all_required_param_names()? {
            let p = if p == "name" { singular(tag) } else { p };
            let n = p.trim_end_matches("_name").trim_end_matches("_id");
            let p = format_ident!("{}", n);

            let formatted = if n == singular(tag) {
                // Format like an argument not a flag.
                format!("[{}]", n)
            } else {
                // TODO: We should give the actual flags here.
                n.to_string()
            };

            let error_msg = format!("{} required in non-interactive mode", formatted);

            required_checks.push(quote!(if #p.is_empty() && !ctx.io.can_prompt() {
                return Err(anyhow::anyhow!(#error_msg));
            }));
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

        // We need to check if project is part of this call for the prompt.
        let project_prompt = if self.is_parameter("project") && tag != "projects" {
            quote! {
                // If they didn't specify a project, prompt for it.
                if project.is_empty() {
                    let mut org_projects: Vec<String> = Vec::new();
                    let projects = client
                        .projects()
                        .get_all(&organization, oxide_api::types::NameOrIdSortMode::NameAscending)
                        .await?;
                    for project in projects {
                        org_projects.push(project.name.to_string());
                    }

                    // Select the project from the selected organization.
                    match dialoguer::Select::new()
                        .with_prompt("Select project:")
                        .items(&org_projects)
                        .interact()
                    {
                        Ok(index) => project = org_projects[index].to_string(),
                        Err(err) => {
                            return Err(anyhow::anyhow!("prompt failed: {}", err));
                        }
                    }
                }
            }
        } else {
            quote!()
        };

        // We need to check if organization is part of this call for the prompt.
        let org_prompt = if self.is_parameter("project") && tag != "projects" {
            quote! {
                // If they didn't specify an organization, prompt for it.
                if organization.is_empty() {
                    let mut orgs: Vec<String> = Vec::new();
                    let resp = client
                        .organizations()
                        .get_all(oxide_api::types::NameOrIdSortMode::NameAscending)
                        .await?;
                    for org in resp {
                        orgs.push(org.name.to_string());
                    }

                    match dialoguer::Select::new()
                        .with_prompt("Project organization:")
                        .items(&orgs)
                        .interact()
                    {
                        Ok(index) => organization = orgs[index].to_string(),
                        Err(err) => {
                            return Err(anyhow::anyhow!("prompt failed: {}", err));
                        }
                    }
                }
            }
        } else {
            quote!()
        };

        let name_prompt = quote!(
            // Prompt for the resource name.
            if #singular_tag_lc.is_empty() {
                match dialoguer::Input::<String>::new()
                    .with_prompt(&format!("{} name:", #singular_tag_str))
                    .interact_text()
                {
                    Ok(name) => #singular_tag_lc = name,
                    Err(err) => {
                        return Err(anyhow::anyhow!("prompt failed: {}", err));
                    }
                }
            }
        );

        let mut additional_prompts: Vec<TokenStream> = Vec::new();
        for p in self.get_all_required_param_names()? {
            let n = p.trim_end_matches("_name").trim_end_matches("_id");
            if n == singular(tag) || n == "project" || n == "organization" || n == "name" {
                // Skip the prompt.
                continue;
            }

            let p = format_ident!("{}", n);

            let title = format!("{} {}:", singular_tag_str, n);

            additional_prompts.push(quote! {
                // Propmt if they didn't provide the value.
                if #p.is_empty() {
                    match dialoguer::Input::<String>::new()
                        .with_prompt(#title)
                        .interact_text()
                    {
                        Ok(desc) => #p = desc,
                        Err(err) => {
                            return Err(anyhow::anyhow!("prompt failed: {}", err));
                        }
                    }
                }
            });
        }

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
                        "{} Created {} {} in {}",
                        cs.success_icon(),
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
                        "{} Created {} {}",
                        cs.success_icon(),
                        #singular_tag_str,
                        full_name
                    )?;
                }
            }
        } else {
            quote! {
                writeln!(
                    ctx.io.out,
                    "{} Created {} {}",
                    cs.success_icon(),
                    #singular_tag_str,
                    self.#singular_tag_lc
                )?;
            }
        };

        let additional_struct_params = self.get_additional_struct_params(tag)?;

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
            }

            #[async_trait::async_trait]
            impl crate::cmd::Command for #struct_name {
                async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                    #(#mutable_variables)*

                    #(#required_checks)*

                    let client = ctx.api_client("")?;

                    // Prompt for various parameters if we can, and the user passed them as empty.
                    #org_prompt

                    #project_prompt

                    #name_prompt

                    #(#additional_prompts)*

                    client
                        .#tag_ident()
                        .post(
                            #(#api_call_params),*
                        )
                        .await?;

                    let cs = ctx.io.color_scheme();
                    #output

                    Ok(())
                }
            }
        );

        let enum_item: syn::Variant = syn::parse2(quote!(Create(#struct_name)))?;

        Ok((cmd, enum_item))
    }

    /// Generate the view command.
    fn generate_view_command(&self, tag: &str) -> Result<(TokenStream, syn::Variant)> {
        let tag_ident = format_ident!("{}", tag);
        let singular_tag_str = if tag == "vpcs" {
            singular(tag).to_uppercase()
        } else {
            singular(tag)
        };
        let singular_tag_lc = format_ident!("{}", singular(tag));
        let struct_name = format_ident!("Cmd{}View", to_title_case(&singular(tag)));

        let struct_doc = format!(
            "View {}.\n\nDisplay information about an Oxide {}.\n\nWith `--web`, open the {} in a web browser instead.",
            singular_tag_str, singular_tag_str, singular_tag_str
        );
        let struct_inner_project_doc = format!("The project that holds the {}.", singular_tag_str);

        let struct_inner_web_doc = format!("Open the {} in the browser.", singular_tag_str);
        let struct_inner_name_doc = format!("The {} to view. Can be an ID or name.", singular_tag_str);

        let api_call_params = self.get_api_call_params(tag)?;

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
                #[doc = #struct_inner_name_doc]
                #[clap(name = #singular_tag_str, required = true)]
                pub #singular_tag_lc: String,

                #project_param

                #organization_param

                #(#additional_struct_params)*

                #[doc = #struct_inner_web_doc]
                #[clap(short, long)]
                pub web: bool,

                // TODO: Change this to be format instead!
                /// Output JSON.
                #[clap(long)]
                pub json: bool,
            }

            #[async_trait::async_trait]
            impl crate::cmd::Command for #struct_name {
                async fn run(&self, ctx: &mut crate::context::Context) -> anyhow::Result<()> {
                    if self.web {
                        // TODO: figure out the right URL.
                        let url = format!(
                            "https://{}/{}",
                            ctx.config.default_host()?,
                            self.#singular_tag_lc
                        );

                        ctx.browser("", &url)?;
                        return Ok(());
                    }

                    let client = ctx.api_client("")?;

                    let result = client.#tag_ident().get(#(#api_call_params),*).await?;

                    if self.json {
                        // If they specified --json, just dump the JSON.
                        ctx.io.write_json(&serde_json::json!(result))?;
                        return Ok(());
                    }

                    let table = tabled::Table::new(vec![result])
                        .with(tabled::Rotate::Left)
                        .with(tabled::Modify::new(tabled::Full)
                            .with(tabled::Alignment::left())
                            .with(tabled::Alignment::top())
                        ).with(tabled::Style::psql().header_off()).to_string();

                    writeln!(ctx.io.out, "{}", table)?;


                    Ok(())
                }
            }
        );

        let enum_item: syn::Variant = syn::parse2(quote!(
                #[clap(alias = "get")]
                View(#struct_name)
        ))?;

        Ok((cmd, enum_item))
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

        let api_call_params = self.get_api_call_params(tag)?;

        let mut api_call_params_all: Vec<TokenStream> = Vec::new();
        for p in self.get_all_param_names()? {
            if p == "limit" || p == "page_token" {
                continue;
            }

            if p == "sort_by" {
                api_call_params_all.push(quote!(self.sort_by.clone()));
                continue;
            }

            let p = format_ident!("{}", p.trim_end_matches("_name").trim_end_matches("_id"));
            api_call_params_all.push(quote!(&self.#p));
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

                let table = tabled::Table::new(results).with(tabled::Style::psql()).to_string();
                writeln!(ctx.io.out, "{}", table)?;

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

        let api_call_params = self.get_api_call_params(tag)?;

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
