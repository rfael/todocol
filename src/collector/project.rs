use log::{info, warn};
use std::{fs::File, io::prelude::*, path::PathBuf};

use super::{comment::Comment, fileformat::FileFormat};

#[derive(Debug)]
pub struct Project {
    pub name:        String,
    pub result_file: PathBuf,
    pub comments:    Vec<Comment>,
}

impl Project {
    pub fn remove_old_result_file(&self) {
        if self.result_file.exists() {
            info!("Removing old result file");
            if let Err(err) = std::fs::remove_file(&self.result_file) {
                warn!("Can not remove old result file: {}", err)
            }
        }
    }

    pub fn save_result_file(&self, format: &FileFormat) -> std::io::Result<()> {
        let mut file = File::create(&self.result_file)?;
        let content = format.format_comments(&self.name, &self.comments);
        file.write_all(&content.into_bytes()[..])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
