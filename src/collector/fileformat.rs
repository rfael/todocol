use log::error;

#[derive(Debug, PartialEq)]
pub enum FileFormat {
    Raw,
    Markdown,
    Json,
}

impl FileFormat {
    pub fn from(t: &str) -> Self {
        match &t.to_string().to_lowercase()[..] {
            "raw" | "txt" => Self::Raw,
            "json" => Self::Json,
            "markdown" | "md" => Self::Markdown,
            t => {
                error!("Outfile type {} not supported, using 'raw' type instead", t);
                Self::default()
            }
        }
    }

    pub fn format_with_extension(&self) -> (Self, &'static str) {
        match self {
            Self::Raw => (Self::Raw, "txt"),
            Self::Json => (Self::Json, "json"),
            Self::Markdown => (Self::Markdown, "md"),
        }
    }

    pub fn file_preamble(&self, title: &str) -> String {
        match self {
            Self::Raw => format!("{}:", title),
            Self::Json => format!("{{\"{}\":[", title),
            Self::Markdown => format!("### {}", title),
        }
    }

    pub fn file_ending(&self) -> String {
        match self {
            Self::Raw => String::new(),
            Self::Json => String::from("]}"),
            Self::Markdown => String::new(),
        }
    }
}

impl Default for FileFormat {
    fn default() -> Self {
        Self::Raw
    }
}

#[cfg(test)]
mod tests {
    use super::FileFormat;
    #[test]
    fn outfile_format_from() {
        assert_eq!(FileFormat::Raw, FileFormat::from("raW"));
        assert_eq!(FileFormat::Json, FileFormat::from("JsOn"));
    }
}
