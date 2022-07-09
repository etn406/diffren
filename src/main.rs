use std::{collections::HashMap};
use std::path::PathBuf;
use colored::*;
use std::fs;
use clap::{Parser, Subcommand};

const DEFAULT_RENAMINGS_FILE: &str = "_renamings.yml";


#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// YAML file listing all the paths to rename.
    /// 
    /// Each line of the file must be a pair `key: value`,
    /// where the key is the path to rename from, and the value is the path to rename to.
    /// Defaults to "_renamings.yml".
    #[clap(parse(from_os_str))]
    input_renamings_file: Option<PathBuf>,

    /// Just verify that each renaming is possible but do not actually rename anything.
    #[clap(short, long, action, default_value_t = false)]
    dry_run: bool,

    /// Read each renaming from the input file in reverse, so the value becomes the path to rename from,
    /// and the key becomes the path to rename to.
    #[clap(long, action, default_value_t = false)]
    revert: bool,

    #[clap(subcommand)]
    commands: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generates a YAML file listing all the files in a directory.
    Generate {
        /// Path of the files to list.
        /// Defaults to the current directory.
        #[clap(parse(from_os_str))]
        from_path: Option<PathBuf>,

        /// YAML file into which write the files list.
        /// Defaults to "_renamings.yml".
        #[clap(parse(from_os_str))]
        output_renamings_file: Option<PathBuf>,
    },
}

enum InputPathError {
    PathDoesntExists,
}

enum OutputPathError {
    PathAlreadyExists,
    PathUsedSeveralTimes(i32),
}

fn main() {
    let args = Args::parse();

    match args.commands {
        Some(command) => match command {
            Commands::Generate {output_renamings_file: renamings_file, from_path} => {
                generate_command(
                    renamings_file.unwrap_or(PathBuf::from(DEFAULT_RENAMINGS_FILE)),
                    from_path.unwrap_or(PathBuf::from(".")),
                )
            }
        },

        // Default command
        None => {
            rename_command(
                args.input_renamings_file.unwrap_or(PathBuf::from(DEFAULT_RENAMINGS_FILE)),
                args.dry_run,
                args.revert
            )
        },
    };

}

fn generate_command(renamings_file: PathBuf, from_path: PathBuf) {
    println!("Gen {:?} from {:?}", renamings_file, from_path)
}

fn rename_command(renamings_file: PathBuf, dry_run: bool, revert: bool) {

    let renamings_file = match std::fs::File::open(renamings_file) {
        Ok(file) => file,
        Err(error) => panic!("{}", format!("Problem opening the file {:?}", error).red()),
    };

    let renamings: HashMap<String, String> = match serde_yaml::from_reader(renamings_file) {
        Ok(renamings) => renamings,
        Err(error) => panic!("{}", format!("Problem parsing the YAML: {:?}", error).red()),
    };

    for (mut input, mut output) in &renamings {
        if revert {
            (input, output) = (output, input);
        }

        let result_input_path = verify_input_path(&input);
        let result_output_path = verify_output_path(&output, &renamings);


        match result_input_path {
            Ok(()) => println!("{}", format!("  from: {:?}", &input)),

            Err(InputPathError::PathDoesntExists) => {
                println!("{}", format!("✗ from: {:?} this path doesn't exist!", &input).red());
            },
        }

        match result_output_path {
            Ok(()) => println!("{}", format!("    to: {:?}", &output)),

            Err(OutputPathError::PathUsedSeveralTimes(count)) => {
                println!("{}", format!("✗   to: {:?} is used {} times, but it should be unique!", &output, count).red());
            },

            Err(OutputPathError::PathAlreadyExists) => {
                println!("{}", format!("✗   to: {:?} this path already exist!", &output).red());
            },
        }

        if matches!(result_input_path, Err(InputPathError::PathDoesntExists)) && matches!(result_output_path, Err(OutputPathError::PathAlreadyExists)) {
            println!("{}", format!("⚠ it seems this path was already renamed.").blue());
        }

        if matches!(result_input_path, Ok(())) && matches!(result_output_path, Ok(())) {
            if !dry_run {
                match fs::rename(input, output) {
                    Ok(()) => println!("{}", "✓ correctly renamed!".green()),
                    Err(_) => println!("{}", "✗ could not be renamed!".red())
                }
            }
        }

        println!();
    }

    if dry_run {
        println!("{}", "Currently in dry run, so no file have been modified.".blue().bold().underline());
        return
    }
}

/// Verify input path
fn verify_input_path(input: &String) -> Result<(), InputPathError> {
    let input_path = PathBuf::from(&input);
    
    if input_path.exists() {
        Ok(())
    } else {
        Err(InputPathError::PathDoesntExists)
    }
}

/// Verify output path
fn verify_output_path(output: &String, hashmap: &HashMap<String, String>) -> Result<(), OutputPathError> {
    let output_path = PathBuf::from(&output);
    let occurrences = count_occurrences(&output, &hashmap);
    
    if output_path.exists() {
        Err(OutputPathError::PathAlreadyExists)
    } else if occurrences > 1 {
        Err(OutputPathError::PathUsedSeveralTimes(occurrences))
    } else {
        Ok(())
    }
}

/// Count the occurences of the same value in the hashmap.
fn count_occurrences(value: &String, hashmap: &HashMap<String, String>) -> i32 {
    let mut count = 0;

    for (_, value2) in hashmap {
        if value2.eq(value) {
            count += 1 
        }
    }

    count
}
