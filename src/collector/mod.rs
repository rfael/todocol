mod comment;
mod fileformat;
mod todofile;

use log::{debug, error, info};
use std::ffi::OsStr;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::path_as_str;
use crate::simple_error_result;
use comment::CommentLine;
use todofile::todofiles_collector;
use todofile::Todofile;

pub use fileformat::FileFormat;

#[derive(Debug)]
pub struct TodoCollector {
    prefixes: Vec<String>,
    comment_symbols: Vec<String>,
    outfile_name: String,
    outfile_format: FileFormat,
    ignored: Vec<String>,
}

impl TodoCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_prefixes(&mut self, prefixes: Vec<String>) {
        self.prefixes = prefixes
    }

    pub fn add_prefix<T: Into<String>>(&mut self, prefix: T) {
        let prefix = prefix.into();
        for p in &self.prefixes {
            if p == &prefix {
                debug!("prefix '{}' already exists", prefix);
                return;
            }
        }
        debug!("prefix '{}' added", prefix);
        self.prefixes.push(prefix)
    }

    pub fn set_comment_symbols(&mut self, comment_symbols: Vec<String>) {
        self.comment_symbols = comment_symbols
    }

    pub fn add_comment_symbol<T: Into<String>>(&mut self, comment_symbol: T) {
        let comment_symbol = comment_symbol.into();
        for p in &self.comment_symbols {
            if p == &comment_symbol {
                debug!("comment symbol '{}' already exists", comment_symbol);
                return;
            }
        }
        debug!("comment symbol '{}' added", comment_symbol);
        self.comment_symbols.push(comment_symbol)
    }

    pub fn set_outfile_name(&mut self, outfile_name: &str) {
        debug!("set outfile name: {}", outfile_name);
        self.outfile_name = String::from(outfile_name);
    }

    pub fn set_outfile_format(&mut self, outfile_format: &str) {
        debug!("set outfile type: {}", outfile_format);
        self.outfile_format = FileFormat::from(outfile_format)
    }

    pub fn set_ignored(&mut self, ignored: Vec<String>) {
        self.ignored = ignored
    }

    pub fn add_ignored<T: Into<String>>(&mut self, ignore: T) {
        let i = ignore.into();
        for p in &self.ignored {
            if p == &i {
                debug!("ignore entry '{}' already exists", i);
                return;
            }
        }
        debug!("ignore entry '{}' added", i);
        self.ignored.push(i)
    }

    /// Checks if all neccesary TodoCollector struct fields are set properly
    pub fn is_valid(&self) -> bool {
        !(self.prefixes.is_empty() || self.comment_symbols.is_empty() || self.outfile_name.is_empty())
    }

    /// check if line contain any of prefixes, and if so prefix is removed from line
    fn comment_to_send(&self, line: &str) -> Option<String> {
        let mut cs = "";
        for c in &self.comment_symbols {
            if line.contains(c) {
                cs = c;
                break;
            }
        }

        if cs.is_empty() {
            return None;
        }

        for p in &self.prefixes {
            if line.contains(p) {
                debug!("Found comment {}", line);
                let mut l = line.trim_start();
                l = l.trim_start_matches(cs);
                l = l.trim_start();
                l = l.trim_start_matches(p);
                l = l.trim_start_matches(':');
                l = l.trim_start();

                debug!("After trimming {}", l);
                return Some(l.to_owned());
            }
        }
        None
    }

    fn handle_file<P: AsRef<Path>>(&self, path: P, todofile: &Todofile) {
        let file_path = PathBuf::from(path.as_ref());
        let file_path_str = path_as_str!(file_path);
        let file_name = file_path.file_name().unwrap_or_else(|| OsStr::new("")).to_str().unwrap_or("");

        let file = match File::open(&file_path) {
            Ok(f) => f,
            Err(err) => {
                error!("Can not open {}: {}", file_path_str, err);
                return;
            }
        };

        debug!("Handling file {}", file_path_str);
        // TODO: skipping binary files
        let reader = BufReader::new(file);

        for (line_num, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(err) => {
                    error!("Reading file {}: {}", file_path_str, err);
                    break;
                }
            };

            if let Some(l) = self.comment_to_send(&line) {
                let comment = CommentLine::new(&l, file_name, line_num);
                if let Err(err) = todofile.save_comment(comment) {
                    error!("Writing '{}' comment failed: {}", l, err);
                }
            }
        }
    }

    fn is_ignored(&self, path: &PathBuf) -> bool {
        match path.file_name() {
            Some(f) => match f.to_str() {
                Some(n) => {
                    for i in &self.ignored {
                        if n == i {
                            return true;
                        }
                    }
                    false
                }
                None => false,
            },
            None => false,
        }
    }

    fn handle_dir_entry(&self, entry: DirEntry, todofile: &Todofile) {
        let entry_name = entry.file_name().into_string().unwrap_or_else(|_| "".to_string());
        let entry_path = entry.path();
        let entry_type = match entry.file_type() {
            Ok(t) => t,
            Err(err) => {
                error!("Can not check type for {}, becouse: {}", path_as_str!(entry_path), err);
                return;
            }
        };

        if self.is_ignored(&entry_path) {
            return;
        }

        if entry_type.is_dir() {
            debug!("Nestet dir in {:?}", entry);
            self.dir_collect(entry_path, todofile);
            return;
        }

        if entry_name.contains(&self.outfile_name) {
            debug!("Skipping outfile {}", path_as_str!(entry_path));
            return;
        }

        self.handle_file(entry_path, &todofile);
    }

    fn dir_collect<P: AsRef<Path>>(&self, dir: P, todofile: &Todofile) {
        let dir = dir.as_ref();
        let entry_iter = match dir.read_dir() {
            Ok(ei) => ei,
            Err(err) => {
                error!("Reading dir {} failed: {}", path_as_str!(dir), err);
                return;
            }
        };

        for e in entry_iter {
            let entry = match e {
                Ok(de) => de,
                Err(_err) => continue,
            };

            self.handle_dir_entry(entry, &todofile);
        }
    }

    /// If Ok returns path to outfile  
    fn project_dir_collect_internal<P: AsRef<Path>>(&self, dir: P) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let project_root = PathBuf::from(dir.as_ref());

        let mut todofile_path = PathBuf::from(&project_root);
        todofile_path.push(&self.outfile_name);

        debug!("todofile: {}", path_as_str!(todofile_path));
        let todofile = Todofile::new(&todofile_path, &self.outfile_format)?;

        self.dir_collect(&project_root, &todofile);

        Ok(todofile.path().to_owned())
    }

    /// Checks project source files to find proper comment and save it into output file
    pub fn project_dir_collect(&self, dir: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if !self.is_valid() {
            error!("prefixes, comment symbols or outfile name not set");
            return simple_error_result!("TodoCollector is not ready");
        }
        info!("collect in project {:?}", &dir);
        let p = Path::new(dir);
        self.project_dir_collect_internal(p)
    }

    /// Check workspace directory for subdirectories witch projects and collcect all project out files
    /// into one out file witch all found comments.
    pub fn workspace_dir_collect<P: AsRef<Path>>(&self, dir: P) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_valid() {
            error!("prefixes, comment symbols or outfile name not set");
            return simple_error_result!("TodoCollector is not ready");
        }

        let ws_root = PathBuf::from(dir.as_ref());
        info!("search in workspace: {:?}", path_as_str!(ws_root));

        let mut created_files: Vec<PathBuf> = Vec::new();
        let ws_entries = fs::read_dir(&ws_root)?;
        // for p in ws_entries {
        //     let project_dir = p?.path();
        //     debug!("Project dir {:?}", project_dir);
        //     match self.project_dir_collect_internal(&project_dir) {
        //         Ok(f) => created_files.push(f),
        //         Err(err) => error!("Can not collect in {} project, because: {}", path_as_str!(project_dir), err),
        //     }
        // }

        let _workspace_todofile = todofiles_collector(&ws_root, &self.outfile_name, &self.outfile_format)?;

        // TODO: Creating one big out file for all workspaces

        Ok(())
    }
}

impl Default for TodoCollector {
    fn default() -> Self {
        Self {
            prefixes: vec![],
            comment_symbols: vec![],
            outfile_name: String::new(),
            outfile_format: FileFormat::default(),
            ignored: vec![],
        }
    }
}

#[cfg(test)]
mod tests {}
