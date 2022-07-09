use std::{collections::HashMap};
use std::path::PathBuf;
use colored::*;
use std::fs;
use serde_derive::{Deserialize};
use clap::{ArgGroup, Parser};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("apply_or_simulate")
        .required(true)
        .args(&["apply", "simulate"]),
))]
struct Args {
    /// Input file
    #[clap(parse(from_os_str))]
    input_file: PathBuf,

    /// Apply the changes
    #[clap(short, long, action)]
    apply: bool,

    /// Simulate the changes
    #[clap(short, long, action)]
    simulate: bool,

    /// Ignore file paths errors when applying changes
    #[clap(long, action, default_value_t = true)]
    ignore_if_already_renamed: bool,

    /// Ignore file paths errors when applying changes
    #[clap(long, action, default_value_t = false)]
    ignore_errors: bool,
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

    let mut error_count = 0;
    let mut changes_to_apply: HashMap<&String, &String> = HashMap::new();

    for (input, output) in &config.rename {
        let result_input_path = verify_input_path(&input);
        let result_output_path = verify_output_path(&output, &config.rename);


        match result_input_path {
            Ok(()) => println!("{}", format!("✓ from: {:?}", &input).green()),

            Err(InputPathError::PathDoesntExists) => {
                println!("{}", format!("✗ from: {:?} this path doesn't exist!", &input).red().bold());
                error_count += 1;
            },
        }

        match result_output_path {
            Ok(()) => println!("{}", format!("✓   to: {:?}", &output).green()),

            Err(OutputPathError::PathUsedSeveralTimes(count)) => {
                println!("{}", format!("✗   to: {:?} is used {} times, but it should be unique!", &output, count).red().bold());
                error_count += 1
            },

            Err(OutputPathError::PathAlreadyExists) => {
                println!("{}", format!("✗   to: {:?} this path already exist!", &output).red().bold());
                error_count += 1
            },
        }

        if matches!(result_input_path, Err(InputPathError::PathDoesntExists)) && matches!(result_output_path, Err(OutputPathError::PathAlreadyExists)) {
            println!("{}", format!("⚠ it seems this path was already renamed.").blue().bold());
            error_count -= 2
        }

        if matches!(result_input_path, Ok(())) && matches!(result_output_path, Ok(())) {
            changes_to_apply.insert(input,output);
        }

        println!();
    }

    if args.simulate {
        println!("{}", "Currently in simulation mode, so no file have been modified.".blue().bold().underline());
        return
    } else if !args.ignore_errors && error_count > 0 {
        let plural = if error_count > 1 { "s" } else { "" };

        println!("{}", format!("{} unexpected error{} detected, so no file have been modified.", error_count, plural).red().bold().underline());

        panic!()
    } else {
        println!("{}", format!("Renaming {:?} file(s)...", &changes_to_apply.len()).blue().bold());

        for (input, output) in &changes_to_apply {
            match fs::rename(input, output) {
                Ok(()) => {
                    println!("{}", format!("✓ renamed {:?}", input).green().bold());
                    println!("{}", format!("       to {:?}", output).green().bold());
                    println!();
                },
                Err(_) => {
                    println!("{}", format!("✗ could not rename {:?}", input).red().bold());
                    println!("{}", format!("                to {:?}", output).red().bold());
                    println!();
                }
            }
        }
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

    for (_, output2) in hashmap {
        if output2.eq(value) {
            count += 1 
        }
    }

    count
}