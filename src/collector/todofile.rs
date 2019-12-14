use log::{debug, error, info};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use super::CommentLine;
use super::FileFormat;
use crate::path_as_str;

enum TodofileCommand {
    Close,
    NotCommand,
}

impl TodofileCommand {
    pub fn command_str(&self) -> String {
        match self {
            Self::Close => String::from("-<CLOSE>-"),
            Self::NotCommand => String::new(),
        }
    }

    pub fn from(line: &str) -> Self {
        match line {
            "-<CLOSE>-" => Self::Close,
            _ => Self::NotCommand,
        }
    }
}

#[derive(Debug)]
pub struct Todofile {
    format: FileFormat,
    path: PathBuf,
    worker_tx: Sender<String>,
    // worker_handle: JoinHandle,
    is_file_opened: bool,
}

impl Todofile {
    pub fn new<P: AsRef<Path>>(path: P, format: &FileFormat) -> Result<Self, Box<dyn std::error::Error>> {
        let mut path = PathBuf::from(path.as_ref());
        let (format, extension) = format.format_with_extension();
        if !path.set_extension(extension) {
            error!("Can not set '{}' extension for file {}", extension, path_as_str!(path));
        }
        let title = title_from_path(&path);
        let file_path_str = path.to_str().unwrap_or("").to_owned();
        info!("Created outfile: {}", file_path_str);

        let (tx, rx) = mpsc::channel::<String>();
        let mut file = File::create(&path)?;

        if let Err(err) = writeln!(file, "{}", format.file_preamble(&title)) {
            error!("can not write to {}: {}", file_path_str, err);
        }
        let file_ending = format.file_ending();
        let _writter = thread::spawn(move || loop {
            let line = match rx.recv() {
                Ok(l) => l,
                Err(_err) => continue,
            };
            debug!("Line received: {}", line);

            match TodofileCommand::from(&line) {
                TodofileCommand::Close => {
                    info!("Closing file {}", file_path_str);
                    if !file_ending.is_empty() {
                        if let Err(err) = writeln!(file, "{}", file_ending) {
                            error!("can not write to {}: {}", file_path_str, err);
                        }
                    }
                    if let Err(err) = file.sync_all() {
                        error!("File {} sync failed: {}", file_path_str, err);
                    }

                    break;
                }
                TodofileCommand::NotCommand => {
                    // TODO: add commas on the end of json format
                    if let Err(err) = writeln!(file, "{}", line) {
                        error!("can not write to {}: {}", file_path_str, err);
                    }
                }
            }
        });

        Ok(Self {
            format,
            path,
            worker_tx: tx,
            // worker_handle: writter,
            is_file_opened: true,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn save_comment(&self, comment: CommentLine) -> Result<(), Box<dyn std::error::Error>> {
        self.worker_tx.send(comment.to_formatted_line(&self.format))?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.worker_tx.send(TodofileCommand::Close.command_str())?;
        // TODO: join writer thread
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.is_file_opened = false;
        Ok(())
    }
}

impl Drop for Todofile {
    fn drop(&mut self) {
        if let Err(err) = self.close() {
            error!("Can not safety close file {}: {}", path_as_str!(self.path), err);
        }
    }
}

fn title_from_path<P: AsRef<Path>>(p: P) -> String {
    let mut path = PathBuf::from(p.as_ref());
    path.pop();
    path.file_name()
        .unwrap_or(&std::ffi::OsStr::new(""))
        .to_str()
        .unwrap_or("")
        .to_string()
}

/// Collect all todofiles from subdirectories (1 level) and writes them in one file
/// If Ok returns path to outfile
///
pub fn todofiles_collector<P: AsRef<Path>>(
    p: P, filename: &str, format: &FileFormat,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = PathBuf::from(p.as_ref());
    let entry_iter = path.read_dir()?;
    path.push(filename);
    let (format, ext) = format.format_with_extension();
    path.set_extension(&ext);
    info!("outfile path: {}", path_as_str!(path));
    let title = title_from_path(&path);

    let mut file = File::create(&path)?;
    writeln!(file, "{}", format.file_preamble(&title))?;

    for e in entry_iter {
        let entry = match e {
            Ok(de) => de,
            Err(_err) => continue,
        };
        debug!("{} workspace entry: {:?}", title, entry);

        let mut tp = entry.path();
        let entry_type = match entry.file_type() {
            Ok(t) => t,
            Err(err) => {
                error!("Can not check type for {}, becouse: {}", path_as_str!(tp), err);
                continue;
            }
        };

        if !entry_type.is_dir() {
            continue;
        }

        tp.push(filename);
        tp.set_extension(&ext);

        let content = match fs::read_to_string(&tp) {
            Ok(c) => c,
            Err(err) => {
                error!("Reading file {} failed: {}", path_as_str!(tp), err);
                continue;
            }
        };

        if let Err(err) = writeln!(file, "{}", content) {
            error!("can not write to {}: {}", path_as_str!(path), err);
        }
    }

    let file_ending = format.file_ending();
    if !file_ending.is_empty() {
        writeln!(file, "{}", file_ending)?;
    }
    file.sync_all()?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::title_from_path;
    use std::path::PathBuf;

    #[test]
    fn todofile_title() {
        let path = PathBuf::from(r"/example/dir/expected/outfile.foo");
        assert_eq!("expected".to_string(), title_from_path(&path));
    }
}
