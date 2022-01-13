use std::{collections::HashMap, io::Read};

use anyhow::{anyhow, Result};
use clap::Parser;

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
}

impl crate::cmd::Command for CmdApi {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        // Let's get the api client.
        let _client = ctx.api_client()?;

        if self.paginate && self.method != http::method::Method::GET {
            return Err(anyhow!("the `--paginate` option is not supported for non-GET requests",));
        }

        // Make sure the endpoint starts with a slash.
        let mut endpoint = self.endpoint.to_string();
        if !self.endpoint.starts_with('/') {
            endpoint = format!("/{}", endpoint);
        }

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

#[cfg(test)]
mod test {

    pub struct TestItem {
        name: String,
        input: String,
        want_out: String,
        want_err: String,
    }

    #[test]
    fn test_cmd_completion_get() {
        let tests = vec![
            TestItem {
                name: "bash completion".to_string(),
                input: "bash".to_string(),
                want_out: "complete -F _oxide -o bashdefault -o default oxide".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "zsh completion".to_string(),
                input: "zsh".to_string(),
                want_out: "#compdef oxide".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "fish completion".to_string(),
                input: "fish".to_string(),
                want_out: "complete -c oxide ".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "PowerShell completion".to_string(),
                input: "powershell".to_string(),
                want_out: "Register-ArgumentCompleter".to_string(),
                want_err: "".to_string(),
            },
            TestItem {
                name: "unsupported shell".to_string(),
                input: "csh".to_string(),
                want_out: "".to_string(),
                want_err: "Invalid variant: csh".to_string(),
            },
        ];

        for _t in tests {}
    }
}
