use colored::Colorize;
use glob::glob;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::validation::Renaming;

/// Directory and temporary files
pub struct TempEditingFiles {
    dir: TempDir,

    /// Temporary file containing current files paths
    pub current: PathBuf,

    /// Temporary file containing the current files paths, meant to be edited
    pub target: PathBuf,
}

/// Create the pair of temporary files necessary for editing.
pub fn init_temporary_files(content: String) -> TempEditingFiles {
    let dir = TempDir::new().expect("Couldn't create temp dir");

    let current = create_temporary_file(&dir, "current names", &content)
        .expect("Couldn't create temp file A");

    let target =
        create_temporary_file(&dir, "target names", &content).expect("Couldn't create temp file B");

    TempEditingFiles {
        dir,
        current,
        target,
    }
}

/// Make sure the temporary files are deleted after use.
pub fn clean_temporary_files(temp: TempEditingFiles) {
    temp.dir.close().expect("Couldn't close the temp dir");
}

/// Creates a temporary file with a content
/// it will be returned (closed) with its path.
fn create_temporary_file(
    parent: &TempDir,
    name: &str,
    content: &String,
) -> Result<PathBuf, std::io::Error> {
    let file_path = parent.path().join(name);
    let mut file = File::create(&file_path)?;
    file.write(content.as_bytes())?;

    Ok(file_path)
}

/// Read a list of paths from a file (or panics if it can't be read).
pub fn read_paths_from(path: &Path) -> Vec<PathBuf> {
    let file = File::open(path).expect(format!("The file {:?} couldn't be read.", path).as_str());
    let buffer = std::io::BufReader::new(file);

    buffer
        .lines()
        .map(|line| {
            let line = line.expect("An error occured reading a line of the file.");
            PathBuf::from(line.trim())
        })
        .collect()
}

// Apply the given renamings and returns the results
pub fn apply_renamings(renamings: Vec<Renaming>) -> Vec<(Renaming, io::Result<()>)> {
    renamings
        .into_iter()
        .map(|renaming| {
            let result = rename(&renaming);
            (renaming, result)
        })
        .collect()
}

// Apply one renaming and returns the result.
fn rename(renaming: &Renaming) -> io::Result<()> {
    let path_to_target = renaming.1.parent();

    // Creates -if necessary- parent folder for the target path.
    if path_to_target.is_some() {
        let path_to_target = path_to_target.unwrap();
        std::fs::create_dir_all(path_to_target)?;
    }

    fs::rename(&renaming.0, &renaming.1)?;

    Ok(())
}

/// Transforms a vector of paths patterns to the corresponding paths list.
pub fn unwrap_paths_patterns(paths: Vec<String>) -> Vec<String> {
    let mut files_paths = vec![];

    for path in paths {
        let is_pattern = path.contains('?') | path.contains('*') | path.contains("**");

        if is_pattern {
            let parsed_paths = glob(path.as_str()).expect("Failed to read pattern");

            for file in parsed_paths {
                match file {
                    Ok(path) => match path.to_str() {
                        Some(path) => files_paths.push(path.to_string()),
                        None => {
                            println!("Couldn't convert to string {}", format!("{:?}", path).red());
                        }
                    },
                    Err(e) => println!("{}", format!("{:?}", e).red()),
                }
            }
        } else {
            files_paths.push(path);
        }
    }

    files_paths.sort();
    files_paths.dedup();

    files_paths
}
