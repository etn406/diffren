use clap::Parser;
use colored::Colorize;
use display::display_results;
use filesystem::{apply_renamings, init_temporary_files, unwrap_paths_patterns};
use interaction::{ask_user_for_changes, NextAction};
use std::process::ExitCode;
use validation::Renaming;

/// Display tables
mod display;

/// File system functions
mod filesystem;

/// Interations with the user
mod interaction;

/// Validation of renamings
mod validation;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path(s) of the files to list.
    /// Unix shell style patterns are supported.
    /// Defaults to `*`.
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

    let temp = init_temporary_files(paths.join("\n"));

    loop {
        match ask_user_for_changes(&temp) {
            NextAction::Confirm(renamings) => {
                let results = apply_renamings(renamings);
                display_results(results);
                break;
            }
            NextAction::Edit => continue,
            NextAction::Exit => {
                println!("\nExiting...");
                break;
            }
        }
    }

    filesystem::clean_temporary_files(temp);

    Ok(())
}
