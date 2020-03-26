#[derive(Debug)]
pub struct Comment {
    content: String,
    source: String,
    line_num: usize,
}

impl Comment {
    pub fn new(comment: &str, source: &str, line_num: usize) -> Self {
        Self {
            content: comment.to_owned(),
            source: source.to_owned(),
            line_num,
        }
    }

    pub fn content(&self) -> &str { &self.content }

    pub fn source(&self) -> &str { &self.source }

    pub fn line_num(&self) -> usize { self.line_num }
}

#[cfg(test)]
mod tests {}
