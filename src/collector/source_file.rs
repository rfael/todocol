use anyhow::anyhow;
use log::{debug, error};
use std::{
    convert::TryFrom,
    fmt,
    fs::File,
    io::{prelude::*, BufReader},
    path::{Path, PathBuf},
};

use super::Comment;

#[derive(Debug, PartialEq)]
pub enum SourceFileType {
    Rust,
    ShellScript,
    C,
    Cpp,
    Python,
}

#[derive(Debug, PartialEq)]
pub struct SourceFile {
    path:        PathBuf,
    source_type: SourceFileType,
}

impl SourceFileType {
    fn comment_symbol(&self) -> &'static str {
        match self {
            Self::Rust | Self::C | Self::Cpp => "//",
            Self::ShellScript | Self::Python => "#",
        }
    }
}

impl SourceFile {
    fn is_comment<'a>(&self, line: &'a str) -> bool {
        line.starts_with(self.source_type.comment_symbol())
    }

    pub fn get_comments(&self) -> anyhow::Result<Vec<Comment>> {
        let mut comments: Vec<Comment> = Vec::new();
        debug!("Searching for comment lines in: {}", self.path.display());
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(err) => {
                    error!("Reading line {} in file {}: {}", line_num, self.path.display(), err);
                    break
                }
            };

            let line = line.trim_start().trim_end();
            if !self.is_comment(line) {
                continue
            }

            let line = line.trim_start_matches(self.source_type.comment_symbol()).trim_start();
            let path_str = self.path.to_str().ok_or_else(|| anyhow!("Parsing path to String failed"))?;
            let comment = Comment::new(line, &path_str, (line_num + 1) as u32);
            comments.push(comment);
        }

        Ok(comments)
    }
}

impl TryFrom<PathBuf> for SourceFile {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let source_type = SourceFileType::try_from(path.as_ref())?;

        Ok(Self { path, source_type })
    }
}

impl TryFrom<&Path> for SourceFileType {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        path.to_str()
            .and_then(|p| {
                // TODO: rewrite extension recognition it using match block
                if p.ends_with(".rs") {
                    Some(Self::Rust)
                } else if p.ends_with(".sh") {
                    Some(Self::ShellScript)
                } else if p.ends_with(".py") {
                    Some(Self::Python)
                } else if p.ends_with(".c") || p.ends_with(".h") {
                    Some(Self::C)
                } else if p.ends_with(".cpp") || p.ends_with(".hpp") {
                    Some(Self::Cpp)
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("File expension not recognized or not supported"))
    }
}

impl fmt::Display for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path() {
        let p = "/foo/bar/baz.rs";
        let path = PathBuf::from(p);
        let expected = SourceFile {
            path:        path.clone(),
            source_type: SourceFileType::Rust,
        };

        let f = SourceFile::try_from(path).unwrap();
        assert_eq!(expected, f)
    }
}
