#[derive(Debug)]
pub struct Comment {
    content:  String,
    source:   String,
    line_num: u32,
}

impl Comment {
    pub fn new(comment: &str, source: &str, line_num: u32) -> Self {
        Self {
            content: comment.to_owned(),
            source: source.to_owned(),
            line_num,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn line_num(&self) -> u32 {
        self.line_num
    }

    pub fn content_append(&mut self, line: &str) {
        self.content.push(' ');
        self.content.push_str(line)
    }
}

#[cfg(test)]
mod tests {}
