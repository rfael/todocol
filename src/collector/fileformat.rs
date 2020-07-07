use log::warn;

use super::Comment;

// TODO: add json and csv format

#[derive(Debug, PartialEq)]
pub enum FileFormat {
    Raw,
    Markdown,
}

impl FileFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::Raw => "txt",
            FileFormat::Markdown => "md",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FileFormat::Raw => "Raw",
            FileFormat::Markdown => "Markdown",
        }
    }

    pub fn format_comments(&self, header: &str, comments: &[Comment]) -> String {
        match self {
            FileFormat::Raw => Self::format_comments_raw(header, comments),
            FileFormat::Markdown => Self::format_comments_markdown(header, comments),
        }
    }

    fn format_comments_raw(header: &str, comments: &[Comment]) -> String {
        let mut result = format!("{}:\n", header);

        for c in comments {
            let comment_line = format!("- {}:{} - {}\n", c.source(), c.line_num(), c.content());
            result.push_str(&comment_line)
        }

        result
    }

    fn format_comments_markdown(header: &str, comments: &[Comment]) -> String {
        let mut result = format!("# {}\n\n", header);

        for c in comments {
            // TODO: prepare format for github
            let comment_line = if let Some(pos) = c.source().find(header) {
                let (_, relative_path) = c.source().split_at(pos);
                format!("* [{}]({}):{} - {} \n", relative_path, c.source(), c.line_num(), c.content())
            } else {
                format!("* [file]({}):{} - {} \n", c.source(), c.line_num(), c.content())
            };
            result.push_str(&comment_line)
        }

        result
    }
}

impl Default for FileFormat {
    fn default() -> Self {
        Self::Raw
    }
}

impl From<&str> for FileFormat {
    fn from(format: &str) -> Self {
        match format.to_string().to_lowercase().as_ref() {
            "raw" | "txt" => Self::Raw,
            "markdown" | "md" => Self::Markdown,
            f => {
                warn!("Outfile format '{}' not supported, using default instead", f);
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {}
