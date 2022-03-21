#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatOutput {
    Json,
    Yaml,
    Table,
}

impl std::fmt::Display for FormatOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            FormatOutput::Json => write!(f, "json"),
            FormatOutput::Yaml => write!(f, "yaml"),
            FormatOutput::Table => write!(f, "table"),
        }
    }
}

impl Default for FormatOutput {
    fn default() -> FormatOutput {
        FormatOutput::Table
    }
}

impl FormatOutput {
    pub fn variants() -> Vec<String> {
        vec!["table".to_string(), "json".to_string(), "yaml".to_string()]
    }
}

impl std::str::FromStr for FormatOutput {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(FormatOutput::Json),
            "yaml" => Ok(FormatOutput::Yaml),
            "table" => Ok(FormatOutput::Table),
            _ => Err(anyhow::anyhow!("Invalid format: {}", s)),
        }
    }
}
