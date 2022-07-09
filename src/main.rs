use std::{collections::HashMap};
use std::path::PathBuf;
use colored::*;
use std::fs;
use serde_derive::{Deserialize};
use clap::{Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[clap(parse(from_os_str))]
    input_file: PathBuf,

    /// Do a dry run
    #[clap(long, action, default_value_t = false)]
    dry_run: bool,

    /// Revert changes
    #[clap(long, action, default_value_t = false)]
    revert: bool,

}

#[derive(Debug, Deserialize)]
struct InputFileFormat {
    rename: HashMap<String, String>,
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

    let config_file = match std::fs::File::open(args.input_file) {
        Ok(file) => file,
        Err(error) => panic!("{}", format!("Problem opening the file {:?}", error).red()),
    };

    let config: InputFileFormat = match serde_yaml::from_reader(config_file) {
        Ok(config) => config,
        Err(error) => panic!("{}", format!("Problem parsing the YAML: {:?}", error).red()),
    };

    for (mut input, mut output) in &config.rename {
        if args.revert {
            (input, output) = (output, input);
        }

        let result_input_path = verify_input_path(&input);
        let result_output_path = verify_output_path(&output, &config.rename);


        match result_input_path {
            Ok(()) => println!("{}", format!("  from: {:?}", &input)),

            Err(InputPathError::PathDoesntExists) => {
                println!("{}", format!("✗ from: {:?} this path doesn't exist!", &input).red().bold());
            },
        }

        match result_output_path {
            Ok(()) => println!("{}", format!("    to: {:?}", &output)),

            Err(OutputPathError::PathUsedSeveralTimes(count)) => {
                println!("{}", format!("✗   to: {:?} is used {} times, but it should be unique!", &output, count).red().bold());
            },

            Err(OutputPathError::PathAlreadyExists) => {
                println!("{}", format!("✗   to: {:?} this path already exist!", &output).red().bold());
            },
        }

        if matches!(result_input_path, Err(InputPathError::PathDoesntExists)) && matches!(result_output_path, Err(OutputPathError::PathAlreadyExists)) {
            println!("{}", format!("⚠ it seems this path was already renamed.").blue().bold());
        }

        if matches!(result_input_path, Ok(())) && matches!(result_output_path, Ok(())) {
            if !args.dry_run {
                match fs::rename(input, output) {
                    Ok(()) => println!("{}", "✓ correctly renamed!".green().bold()),
                    Err(_) => println!("{}", "✗ could not be rename!".red().bold())
                }
            }
        }

        println!();
    }

    if args.dry_run {
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
