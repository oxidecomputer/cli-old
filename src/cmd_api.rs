use std::{
    collections::HashMap,
    io::{Read, Write},
};

use anyhow::{anyhow, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

/// Makes an authenticated HTTP request to the Oxide API and prints the response.
///
/// The endpoint argument should be a path of a Oxide API endpoint.
///
/// The default HTTP request method is "GET" normally and "POST" if any parameters
/// were added. Override the method with `--method`.
///
/// Pass one or more `-f/--raw-field` values in "key=value" format to add static string
/// parameters to the request payload. To add non-string or otherwise dynamic values, see
/// `--field` below. Note that adding request parameters will automatically switch the
/// request method to POST. To send the parameters as a GET query string instead, use
/// `--method GET`.
///
/// The `-F/--field` flag has magic type conversion based on the format of the value:
///
/// - literal values "true", "false", "null", and integer/float numbers get converted to
///   appropriate JSON types;
/// - if the value starts with "@", the rest of the value is interpreted as a
///   filename to read the value from. Pass "-" to read from standard input.
///
/// Raw request body may be passed from the outside via a file specified by `--input`.
/// Pass "-" to read from standard input. In this mode, parameters specified via
/// `--field` flags are serialized into URL query parameters.
///
/// In `--paginate` mode, all pages of results will sequentially be requested until
/// there are no more pages of results.
#[derive(Parser, Debug, Clone)]
#[clap(verbatim_doc_comment)]
pub struct CmdApi {
    /// The endpoint to request.
    #[clap(name = "endpoint", required = true)]
    pub endpoint: String,

    /// The HTTP method for the request.
    #[clap(short = 'X', long, default_value = "GET")]
    pub method: http::method::Method,

    /// Make additional HTTP requests to fetch all pages of results.
    #[clap(long, conflicts_with = "input")]
    pub paginate: bool,

    /// Add a typed parameter in key=value format.
    #[clap(short = 'F', long)]
    pub field: Vec<String>,

    /// Add a string parameter in key=value format.
    #[clap(short = 'f', long)]
    pub raw_field: Vec<String>,

    /// The file to use as body for the HTTP request (use "-" to read from standard input).
    #[clap(long, default_value = "", conflicts_with = "paginate")]
    pub input: String,

    /// Include HTTP response headers in the output.
    #[clap(short, long)]
    pub include: bool,
}

/// The JSON type for a paginated response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginatableResponse {
    /// The items in the response.
    pub items: Vec<serde_json::Value>,
    /// The pagination information for the response.
    pub next_page: Option<String>,
}

#[async_trait::async_trait]
impl crate::cmd::Command for CmdApi {
    async fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        // Let's get the api client.
        let client = ctx.api_client("")?;

        if self.paginate && self.method != http::method::Method::GET {
            return Err(anyhow!("the `--paginate` option is not supported for non-GET requests",));
        }

        // Make sure the endpoint starts with a slash.
        let mut endpoint = self.endpoint.to_string();
        if !self.endpoint.starts_with('/') {
            endpoint = format!("/{}", endpoint);
        }

        // Parse the fields.
        let params = self.parse_fields()?;

        // Set them as our body if they exist.
        let mut b = String::new();
        if !params.is_empty() {
            b = serde_json::to_string(&params)?;
        }

        let mut bytes = b.as_bytes().to_vec();

        // TODO: If they didn't specify the method and we have parameters, we'll
        // assume they want to use POST.

        // Parse the input file.
        if !self.input.is_empty() {
            // Read the input file.
            let mut buf = Vec::new();
            let mut input_file = std::fs::File::open(&self.input)?;
            input_file.read_to_end(&mut buf)?;

            // Set this as our body.
            bytes = buf.clone();

            // Set our params to the query string.
            if !params.is_empty() {
                let mut query_string = String::new();
                for (key, value) in params {
                    if !query_string.is_empty() {
                        query_string.push('&');
                    }
                    query_string.push_str(&format!("{}={}", key, value));
                }

                endpoint = add_query_string(&endpoint, &query_string);
            }
        }

        // Make the request.
        let mut has_next_page = true;
        let mut result = serde_json::Value::Null;
        let mut page_results: Vec<serde_json::Value> = Vec::new();
        while has_next_page {
            let body = if bytes.is_empty() {
                None
            } else {
                Some(reqwest::Body::from(bytes.clone()))
            };

            // TODO: We could also add flags for setting headers, etc.
            let req = client.request_raw(self.method.clone(), &endpoint, body).await?;
            let resp = req.send().await?;

            // Print the response headers if requested.
            if self.include {
                writeln!(ctx.io.out, "{:?} {}", resp.version(), resp.status())?;
                print_headers(ctx, resp.headers())?;
            }

            if resp.status() == 204 {
                return Ok(());
            }

            if !resp.status().is_success() {
                return Err(anyhow!(
                    "{} {}",
                    resp.status(),
                    resp.status().canonical_reason().unwrap_or("")
                ));
            }

            if self.paginate {
                let mut page: PaginatableResponse = resp.json().await?;

                if !page.items.is_empty() {
                    page_results.append(&mut page.items);
                }

                match page.next_page {
                    Some(next_page) => {
                        endpoint = add_query_string(&endpoint, &format!("page_token={}", next_page));
                    }
                    None => {
                        has_next_page = false;
                    }
                }
            } else {
                // Read the response body.
                result = resp.json().await?;
                has_next_page = false;
            }
        }

        if self.paginate {
            result = serde_json::Value::Array(page_results);
        }

        ctx.io.write_json(&result)?;

        Ok(())
    }
}

impl CmdApi {
    fn parse_fields(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut params: HashMap<String, serde_json::Value> = HashMap::new();

        // Parse the raw fields.
        // These are always added as strings.
        for f in self.raw_field.iter() {
            let mut parts = f.splitn(2, '=');
            let key = parts.next().ok_or_else(|| anyhow!("missing key in --raw-field"))?;
            let value = parts.next().ok_or_else(|| anyhow!("missing value in --raw-field"))?;

            params.insert(key.to_string(), serde_json::Value::String(value.to_string()));
        }

        // Parse the typed fields.
        for t in self.field.iter() {
            let mut parts = t.splitn(2, '=');
            let key = parts.next().ok_or_else(|| anyhow!("missing key in --field"))?;
            let value = parts.next().ok_or_else(|| anyhow!("missing value in --field"))?;

            // See if value parses as an integer.
            if let Ok(i) = value.parse::<i64>() {
                params.insert(key.to_string(), serde_json::Value::Number(i.into()));
                continue;
            }

            // See if value parses as a float.
            if let Ok(f) = value.parse::<f64>() {
                let num = serde_json::Number::from_f64(f).ok_or_else(|| anyhow!("invalid float {}", f))?;
                params.insert(key.to_string(), serde_json::Value::Number(num));
                continue;
            }

            // Check the rest.
            let value = match value {
                "true" => serde_json::Value::Bool(true),
                "false" => serde_json::Value::Bool(false),
                "null" => serde_json::Value::Null,
                _ => {
                    // Check if we have a file.
                    if value.starts_with('@') {
                        let filename = value.trim_start_matches('@');
                        let mut file = std::fs::File::open(filename)?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;
                        serde_json::Value::String(contents)
                    } else if value == "-" {
                        // Read from stdin.
                        let mut contents = String::new();
                        std::io::stdin().read_to_string(&mut contents)?;
                        serde_json::Value::String(contents)
                    } else {
                        serde_json::Value::String(value.to_string())
                    }
                }
            };

            params.insert(key.to_string(), value);
        }

        Ok(params)
    }
}

fn print_headers(ctx: &mut crate::context::Context, headers: &reqwest::header::HeaderMap) -> Result<()> {
    let mut names: Vec<String> = headers.keys().map(|k| k.as_str().to_string()).collect();
    names.sort_by_key(|a| a.to_lowercase());

    let cs = ctx.io.color_scheme();

    let mut tw = tabwriter::TabWriter::new(vec![]);
    for name in names {
        if name.to_lowercase() == "status" {
            continue;
        }

        let value = headers.get(name.as_str()).unwrap();

        writeln!(tw, "{}:\t{:?}\n", cs.cyan(&name), value)?;
    }

    tw.flush()?;

    let table = String::from_utf8(tw.into_inner()?)?;
    writeln!(ctx.io.out, "{}", table)?;

    Ok(())
}

fn add_query_string(endpoint: &str, query_string: &str) -> String {
    if endpoint.contains('?') {
        format!("{}&{}", endpoint, query_string)
    } else {
        format!("{}?{}", endpoint, query_string)
    }
}
