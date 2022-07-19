use clap::Parser;
use colored::Colorize;
use edit::PathChange;
use glob::glob;
use std::{fs, path::PathBuf, process::ExitCode};
mod edit;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path(s) of the files to list.
    /// Unix shell style patterns are supported, for example "mymusic/**/*.mp3".
    /// Defaults to all the files in the current directory.
    #[clap(value_parser)]
    paths: Vec<String>,
}

fn main() -> ExitCode {
    match exec() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            println!("{}", message.bright_red());
            ExitCode::FAILURE
        }
    }
}

fn exec() -> Result<(), String> {
    let args = Args::parse();

    let paths = args.paths;

    // Default to "*" all files in the current directory
    let paths = if paths.len() == 0 {
        vec!["*".to_string()]
    } else {
        paths
    };

    let paths = unwrap_paths_patterns(paths);

    let temp = edit::init_temp_files(paths.join("\n"));

    loop {
        match edit::ask_user_for_changes(&temp) {
            edit::NextAction::Confirm(changes) => {
                rename_paths_and_display(changes);
                break;
            }
            edit::NextAction::Edit => continue,
            edit::NextAction::Exit => {
                println!("\nExiting...");
                break;
            }
        }
    }

    edit::clean_temp_files(temp);

    Ok(())
}

/// Transforms a vector of paths patterns to the corresponding paths list.
fn unwrap_paths_patterns(paths: Vec<String>) -> Vec<String> {
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

fn rename_paths_and_display(changes: Vec<PathChange>) {
    let to_str = |p: PathBuf| p.to_str().unwrap_or("?").normal();

    edit::display_table(
        changes
            .into_iter()
            .map(|(current, target)| match rename_path(&current, &target) {
                Ok(()) => [to_str(current), to_str(target), "✓ Renamed".green()],
                Err(e) => [to_str(current), to_str(target), format!("✗ {}", e).red()],
            })
            .collect(),
        "→",
    );
}

fn rename_path(current: &PathBuf, target: &PathBuf) -> std::io::Result<()> {
    let path_to_target = target.parent();

    if path_to_target.is_some() {
        let path_to_target = path_to_target.unwrap();
        std::fs::create_dir_all(path_to_target)?;
    }

    fs::rename(&current, &target)?;

    Ok(())
}
