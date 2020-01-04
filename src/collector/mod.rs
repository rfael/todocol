mod comment;
mod fileformat;

use log::{debug, error, info, warn};
use simple_error::SimpleError;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub use comment::Comment;
pub use fileformat::FileFormat;

#[derive(Debug, Default)]
pub struct TodoCollector {
    prefixes: Vec<String>,
    comment_symbols: Vec<String>,
    source_extensions: Vec<String>,
    outfile_name: String,
    outfile_format: FileFormat,
    ignore: Vec<String>,
    source_files: Vec<PathBuf>,
    outfile: PathBuf,
    comments: Vec<Comment>,
    project_name: String,
}

// TODO: use comment symbols associated witch extension

impl TodoCollector {
    pub fn new() -> Self {
        Self {
            prefixes: vec!["TODO".into()],
            comment_symbols: vec!["//".into()],
            source_extensions: vec![".rs".into()],
            outfile_name: String::from("TODO"),
            outfile_format: FileFormat::default(),
            ignore: Vec::new(),
            source_files: Vec::new(),
            outfile: PathBuf::default(),
            comments: Vec::new(),
            project_name: String::default(),
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
            return;
        }

        for p in &self.prefixes {
            if p == &prefix {
                debug!("prefix '{}' already exists", prefix);
                return;
            }
        }
        info!("prefix '{}' added", prefix);
        self.prefixes.push(prefix)
    }

    pub fn set_comment_symbols(&mut self, comment_symbols: Vec<String>) {
        if !comment_symbols.is_empty() {
            self.comment_symbols = comment_symbols
        }
    }

    pub fn add_comment_symbol<T: Into<String>>(&mut self, comment_symbol: T) {
        let comment_symbol = comment_symbol.into();

        if comment_symbol.is_empty() {
            return;
        }

        for p in &self.comment_symbols {
            if p == &comment_symbol {
                debug!("comment symbol '{}' already exists", comment_symbol);
                return;
            }
        }
        info!("comment symbol '{}' added", comment_symbol);
        self.comment_symbols.push(comment_symbol)
    }

    pub fn set_source_extensions(&mut self, source_extensions: Vec<String>) {
        if !source_extensions.is_empty() {
            self.source_extensions = source_extensions
        }
        self.remove_extensions_dots();
    }

    pub fn add_source_extension<T: Into<String>>(&mut self, source_extension: T) {
        let source_extension = source_extension.into();

        if source_extension.is_empty() {
            return;
        }

        for p in &self.source_extensions {
            if p == &source_extension {
                debug!("comment symbol '{}' already exists", source_extension);
                return;
            }
        }
        info!("comment symbol '{}' added", source_extension);
        self.source_extensions.push(source_extension);
        self.remove_extensions_dots();
    }

    pub fn set_outfile_name(&mut self, outfile_name: &str) {
        self.outfile_name = String::from(outfile_name);
        info!("outfile name: {}", outfile_name);
    }

    pub fn set_outfile_format(&mut self, outfile_format: &str) {
        self.outfile_format = FileFormat::from(outfile_format);
        info!("outfile type: {}", self.outfile_format.as_str());
    }

    pub fn set_ignore(&mut self, ignored: Vec<String>) {
        self.ignore = ignored
    }

    pub fn add_ignore<T: Into<String>>(&mut self, ignore: T) {
        let i = ignore.into();

        if i.is_empty() {
            return;
        }

        for p in &self.ignore {
            if p == &i {
                debug!("ignore entry '{}' already exists", i);
                return;
            }
        }
        info!("ignore entry '{}' added", i);
        self.ignore.push(i)
    }

    /// Checks if all necessary TodoCollector struct fields are set properly
    pub fn is_valid(&self) -> bool {
        !(self.prefixes.is_empty()
            || self.comment_symbols.is_empty()
            || self.outfile_name.is_empty()
            || self.source_extensions.is_empty())
    }

    fn remove_extensions_dots(&mut self) {
        self.source_extensions = self.source_extensions.iter().map(|e| e.replace(".", "")).collect()
    }

    fn files_to_check<P: AsRef<Path>>(&self, dir: P) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let entry_iter = match dir.as_ref().read_dir() {
            Ok(ei) => ei,
            Err(err) => {
                warn!("Can not read dir {:?}: {}", dir.as_ref(), err);
                return result;
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
                    break;
                }
            }

            if skip {
                continue;
            }

            if path.is_dir() {
                let r = self.files_to_check(&path);
                result.extend(r);
                continue;
            }

            let mut ext_ok = false;
            for ext in &self.source_extensions {
                if let Some(e) = path.extension().and_then(|e| e.to_str()) {
                    if ext == e {
                        ext_ok = true;
                    }
                }
            }

            if ext_ok {
                result.push(path);
            }
        }

        result
    }

    fn is_comment<'a>(&self, line: &'a str) -> Option<&'a str> {
        let line = line.trim_start();
        for cs in &self.comment_symbols {
            if line.starts_with(cs) {
                for p in &self.prefixes {
                    let prefix = format!("{} {}:", cs, p);
                    if let Some(pos) = line.find(&prefix) {
                        let (_, comment) = line.split_at(pos + prefix.len());
                        return Some(comment.trim_start());
                    }
                }
            }
        }
        None
    }

    /// Chose dir to collect comments, this function prepares list of source files to check, creates path
    /// to new todo collected file, removes old one if it exist, and sets project name
    pub fn set_project_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SimpleError> {
        let project_dir = path.as_ref();
        let mut outfile_path = PathBuf::from(project_dir);
        self.project_name = match outfile_path.file_name() {
            Some(pn) => match pn.to_str() {
                Some(n) => n.into(),
                None => return Err(SimpleError::new("Parse OsStr to &str failed")),
            },
            None => return Err(SimpleError::new("Can not get project name from path")),
        };

        outfile_path.push(&self.outfile_name);
        outfile_path.set_extension(self.outfile_format.extension());

        let outfile_path_str = match outfile_path.to_str() {
            Some(p) => p,
            None => return Err(SimpleError::new("Can not parse outfile path to &str")),
        };

        info!("todo file: {}", outfile_path_str);

        if outfile_path.exists() {
            info!("Removing old todo file");
            if let Err(err) = std::fs::remove_file(&outfile_path) {
                warn!("Can not remove {}: {}", outfile_path_str, err)
            }
        }

        self.source_files = self.files_to_check(path);
        self.outfile = outfile_path;

        Ok(())
    }

    /// Collects comments from source file list
    pub fn collect_project(&mut self) {
        info!("Collecting todo comments");

        for source_file in &self.source_files {
            let source_file_str = match source_file.to_str() {
                Some(f) => f,
                None => continue,
            };

            info!("Checking in {}", source_file_str);
            let file = match File::open(&source_file) {
                Ok(f) => f,
                Err(err) => {
                    error!("Can not open {}: {}", source_file_str, err);
                    return;
                }
            };

            let reader = BufReader::new(file);
            for (line_num, line) in reader.lines().enumerate() {
                let line = match line {
                    Ok(l) => l,
                    Err(err) => {
                        error!("Reading line {} in file {}: {}", line_num, source_file_str, err);
                        break;
                    }
                };

                if let Some(comment_line) = self.is_comment(&line) {
                    debug!("todo comment: {}", comment_line);
                    self.comments.push(Comment::new(comment_line, source_file_str, line_num))
                }
            }
        }
    }

    /// Saves collected comments to file and clear them if save were ok
    pub fn save_comments(&mut self) -> std::io::Result<()> {
        let mut file = File::create(&self.outfile)?;
        let content = self.outfile_format.format_comments(&self.project_name, &self.comments);
        file.write_all(&content.into_bytes()[..])?;
        self.comments.clear();
        Ok(())
    }
}
