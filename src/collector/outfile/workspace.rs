use log::{debug, error, info};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use super::OutfileFormat;
use crate::simple_error_result;

#[derive(Debug)]
pub struct OutfileWorkspace {
    format: OutfileFormat,
    path: PathBuf,
    name: String,
    level: u8,
}

impl OutfileWorkspace {
    pub fn merge_project_outfiles<P: AsRef<Path>>(
        files: &Vec<PathBuf>, workspace: P,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut result_path = PathBuf::from(workspace.as_ref());
        let workspace_name = file_name_from_path(&result_path)?;
        let (outfile_name, extension) = match &files.get(0) {
            Some(p) => (file_name_from_path(&p)?, file_extension_from_path(&p)?),
            None => return simple_error_result!("merge_out_files failed: empty files vector"),
        };
        let format = OutfileFormat::from(&extension);

        result_path.push(&outfile_name);
        result_path.set_extension(&extension);
        debug!("Merging project outfiles into {:?}", result_path);

        // TODO: merging files

        Ok(result_path)
    }
}

// TODO: move file_extension_from_path, file_name_from_path functions somewhere else
fn file_name_from_path<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let p = PathBuf::from(path.as_ref());
    // let err_result = simple_error_result!("Reading file name from path {} failed", &p.to_str().unwrap_or_else(|| ""));
    // match p.file_name() {
    //     Some(f) => match f.to_str() {
    //         Some(n) => Ok(n.to_string()),
    //         None => err_result,
    //     },
    //     None => err_result,
    // }
    Ok(String::new())
}

fn file_extension_from_path<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn std::error::Error>> {
    let p = PathBuf::from(path.as_ref());
    // let err = simple_error_result!(
    //     "Reading file extension from path {} failed",
    //     &p.to_str().unwrap_or_else(|| "")
    // );
    // match p.extension() {
    //     Some(f) => match f.to_str() {
    //         Some(n) => Ok(n.to_string()),
    //         None => Err(Box::new(err)),
    //     },
    //     None => Err(Box::new(err)),
    // }
    Ok(String::new())
}

#[cfg(test)]
mod tests {}

// pub fn merge_out_files<P: AsRef<Path>>(files: &Vec<PathBuf>, workspace: P) -> Result<PathBuf, Box<dyn std::error::Error>> {
//     let mut m_path = PathBuf::from(workspace.as_ref());
//     let workspace_name = &m_path
//         .file_name()
//         .unwrap_or_else(|| OsStr::new("_workspace_"))
//         .to_str()
//         .unwrap_or("__workspace__")
//         .to_string();
//     let (file_name, extension) = match &files.get(0) {
//         Some(p) => (p.file_name(), p.extension()),
//         None => return Err(Box::new(Error::new("merge_out_files failed: empty files vector"))),
//     };

//     let file_name = match file_name {
//         Some(p) => match p.to_str() {
//             Some(s) => s,
//             None => return Err(Box::new(Error::new("merge_out_files failed: can not read output file name"))),
//         },
//         None => return Err(Box::new(Error::new("merge_out_files failed: can not read output file name"))),
//     };

//     let extension = extension.unwrap_or_else(|| OsStr::new("")).to_str().unwrap_or("txt");
//     let format = Self::file_format_from_extension(extension);
//     m_path.push(file_name);
//     m_path.set_extension(extension);
//     debug!("Merging project outfiles into {:?}", m_path);

//     let mut w_file = File::create(&m_path)?;

//     match format {
//         OutfileFormat::Raw => writeln!(w_file, "{}:", &workspace_name)?,
//         OutfileFormat::Json => writeln!(w_file, "{{ \"{}\":[", &workspace_name)?,
//         OutfileFormat::Markdown => writeln!(w_file, "## {}", &workspace_name)?,
//     }

//     for f in files {
//         let p_file = match File::open(&f) {
//             Ok(p) => p,
//             Err(err) => {
//                 error!("can not open {}: {}", f.to_str().unwrap_or("__out_file__"), err);
//                 continue;
//             }
//         };

//         let reader = BufReader::new(p_file);
//         for (line_num, line) in reader.lines().enumerate() {
//             let line = match line {
//                 Ok(l) => l,
//                 Err(err) => {
//                     error!("Reading file {}: {}", f.to_str().unwrap_or("__out_file__"), err);
//                     break;
//                 }
//             };

//             let wr = match format {
//                 OutfileFormat::Raw => writeln!(w_file, "{}", line),
//                 OutfileFormat::Json => {
//                     if line_num != 0 {
//                         write!(w_file, "{},", line)
//                     } else {
//                         if let Err(err) = writeln!(w_file, ",") {
//                             Err(err)
//                         } else {
//                             write!(w_file, "{},", line)
//                         }
//                     }
//                 }
//                 OutfileFormat::Markdown => writeln!(w_file, "{}", line),
//             };

//             if let Err(err) = wr {
//                 error!("can not write {}: {}", m_path.to_str().unwrap_or("__out_file__"), err);
//                 continue;
//             }
//         }
//     }

//     match format {
//         OutfileFormat::Raw => (),
//         OutfileFormat::Json => writeln!(w_file, "\n]}}")?,
//         OutfileFormat::Markdown => (),
//     }

//     Ok(m_path)
// }
