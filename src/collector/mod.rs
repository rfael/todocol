mod comment;
mod fileformat;
mod project;
mod source_file;

use anyhow::anyhow;
use log::{debug, error, info, warn};
use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
};

use project::Project;
use source_file::SourceFile;

pub use comment::Comment;
pub use fileformat::FileFormat;

#[derive(Debug, Default)]
pub struct TodoCollector {
    prefixes:      Vec<String>,
    ignore:        Vec<String>,
    result_name:   String,
    result_format: FileFormat,
}

// TODO: add TodoCollectorBuilder struct

impl TodoCollector {
    pub fn new() -> Self {
        Self {
            prefixes:      vec!["TODO".into()],
            ignore:        Vec::new(),
            result_name:   String::from("TODO"),
            result_format: FileFormat::default(),
        }
    }

    pub fn set_prefixes(&mut self, prefixes: Vec<String>) {
        if !prefixes.is_empty() {
            self.prefixes = prefixes
        }
    }

    pub fn add_prefix<T: Into<String>>(&mut self, prefix: T) {
        let prefix = prefix.into();

        if prefix.is_empty() {
            return
        }

        for p in &self.prefixes {
            if p == &prefix {
                debug!("prefix '{}' already exists", prefix);
                return
            }
        }
        info!("prefix '{}' added", prefix);
        self.prefixes.push(prefix)
    }

    pub fn set_result_file_name(&mut self, name: &str) {
        if name.is_empty() {
            error!("Result file name can not be empty");
            return
        }
        info!("Result file name: {}", name);
        self.result_name = String::from(name);
    }

    pub fn set_result_file_format(&mut self, format: &str) {
        self.result_format = FileFormat::from(format);
        info!("Result file type: {}", self.result_format.as_str());
    }

    pub fn set_ignore(&mut self, ignored: Vec<String>) {
        self.ignore = ignored
    }

    pub fn add_ignore<T: Into<String>>(&mut self, ignore: T) {
        let i = ignore.into();

        if i.is_empty() {
            return
        }

        for p in &self.ignore {
            if p == &i {
                debug!("ignore entry '{}' already exists", i);
                return
            }
        }
        info!("ignore entry '{}' added", i);
        self.ignore.push(i)
    }

    fn get_source_files<P: AsRef<Path>>(&self, dir: P) -> Vec<SourceFile> {
        let mut result = Vec::new();
        let entry_iter = match dir.as_ref().read_dir() {
            Ok(ei) => ei,
            Err(err) => {
                warn!("Can not read dir {:?}: {}", dir.as_ref(), err);
                return result
            }
        };

        for entry in entry_iter {
            let entry = match entry {
                Ok(e) => e,
                Err(_err) => continue,
            };

            let path = entry.path();
            let mut skip = false;
            for ign in &self.ignore {
                if path.ends_with(ign) {
                    skip = true;
                    break
                }
            }

            if skip {
                continue
            }

            if path.is_dir() {
                let r = self.get_source_files(&path);
                result.extend(r);
                continue
            }

            if let Ok(s) = SourceFile::try_from(path) {
                result.push(s)
            }
        }

        result
    }

    fn get_result_file_path<P: AsRef<Path>>(&self, project_dir: P) -> anyhow::Result<PathBuf> {
        let mut path = PathBuf::from(project_dir.as_ref());

        if self.result_name.is_empty() {
            return Err(anyhow!("Result file name is empty"))
        }

        path.push(&self.result_name);
        path.set_extension(self.result_format.extension());

        let result_file_path = path.to_str().ok_or_else(|| anyhow!("Preparing result file path failed"))?;
        info!("Result file path: {}", result_file_path);
        Ok(path)
    }

    /// Can be used only for comments from same file
    fn comments_filter(&self, comments: Vec<Comment>) -> Vec<Comment> {
        let mut last_ok_line = 0;
        let mut ret: Vec<Comment> = Vec::new();

        for c in comments {
            let mut ok = false;
            for p in &self.prefixes {
                if c.content().starts_with(p) {
                    ok = true;
                    break
                }
            }

            if ok {
                last_ok_line = c.line_num();
                ret.push(c);
            } else if (last_ok_line + 1) == c.line_num() && !ret.is_empty() {
                let last = ret.len() - 1;
                ret[last].content_append(c.content());
                last_ok_line = c.line_num();
            }
        }

        ret
    }

    pub fn collect_from_project<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let project_dir = PathBuf::from(path.as_ref());
        let project_name = project_dir
            .file_name()
            .map(|pn| pn.to_string_lossy())
            .ok_or_else(|| anyhow!("Can not get project name from path"))?;
        let result_file_path = self.get_result_file_path(&project_dir)?;
        let source_files = self.get_source_files(&project_dir);
        let mut comments = Vec::new();

        info!("project name: {}", project_name);

        // TODO: use rayon to concurrent collecting
        for source_file in source_files {
            if let Ok(c) = source_file.get_comments() {
                let mut c = self.comments_filter(c);
                if !c.is_empty() {
                    comments.append(&mut c)
                }
            }
        }

        let project = Project {
            name: project_name,
            result_file: result_file_path,
            comments,
        };

        project.remove_old_result_file();
        project.save_result_file(&self.result_format)?;

        Ok(())
    }
}
