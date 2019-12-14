use super::FileFormat;

pub struct CommentLine {
    content: String,
    file_name: String,
    line_num: usize,
}

impl CommentLine {
    pub fn new(comment: &str, file_name: &str, line_num: usize) -> Self {
        Self {
            content: comment.to_owned(),
            file_name: file_name.to_owned(),
            line_num,
        }
    }

    pub fn to_formatted_line(&self, format: &FileFormat) -> String {
        match format {
            FileFormat::Raw => format!("- {}[{}]: {}", self.file_name, self.line_num, self.content),
            FileFormat::Json => format!(
                "{{\"file\": \"{}\", \"line\": \"{}\", \"comment\": \"{}\" }}",
                self.file_name, self.line_num, self.content
            ),
            FileFormat::Markdown => format!("* **{}** {}: {}", self.file_name, self.line_num, self.content),
        }
    }
}

impl Default for CommentLine {
    fn default() -> Self {
        Self {
            content: String::new(),
            file_name: String::new(),
            line_num: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CommentLine;
    use super::FileFormat;
    #[test]
    fn comment_as_formated_line() {
        let cl = CommentLine::new("ciamciaramcia", "system32", 7);
        assert_eq!("- system32[7]: ciamciaramcia", cl.to_formatted_line(&FileFormat::Raw));
        assert_eq!(
            "{\"file\": \"system32\", \"line\": \"7\", \"comment\": \"ciamciaramcia\" }",
            cl.to_formatted_line(&FileFormat::Json)
        );
        assert_eq!("* **system32** 7: ciamciaramcia", cl.to_formatted_line(&FileFormat::Markdown));
    }
}
