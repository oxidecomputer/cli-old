use anyhow::Result;
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
/// - literal values "true", "false", "null", and integer numbers get converted to
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
    pub method: String,

    /// Make additional HTTP requests to fetch all pages of results.
    #[clap(long)]
    pub paginate: bool,

    /// Add a typed parameter in key=value format.
    #[clap(short = 'F', long)]
    pub field: Vec<String>,

    /// Add a string parameter in key=value format.
    #[clap(short = 'f', long)]
    pub raw_field: Vec<String>,

    /// The file to use as body for the HTTP request (use "-" to read from standard input).
    #[clap(long, default_value = "")]
    pub input: String,
}

impl crate::cmd::Command for CmdApi {
    fn run(&self, ctx: &mut crate::context::Context) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

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

        for t in tests {}
    }
}
