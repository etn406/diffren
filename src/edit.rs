use colored::{ColoredString, Colorize};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::cmp::max;
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Path Change Result
type PathChangeResult = Result<(), PathChangeError>;

/// Path Change Error
enum PathChangeError {
    InputDoesntExist,
    OutputAlreadyExists,
    SeveralOutputsAreTheSame(u32),
    InvalidOutputPath,
    InvalidInputPath,
    Unchanged,
}

/// Path renaming with input and output
pub type PathChange = (PathBuf, PathBuf);

/// Next action to perform :
pub enum NextAction {
    /// Confirm the changes
    Confirm(Vec<PathChange>),
    Edit,
    Exit,
}

/// Directory and temporary files
pub struct TempEditingFiles {
    dir: TempDir,
    input: PathBuf,
    output: PathBuf,
}

pub fn init_temp_files(content: String) -> TempEditingFiles {
    let dir = TempDir::new().expect("Couldn't create temp dir");

    let input =
        make_temp_file(&dir, "current names", &content).expect("Couldn't create temp file A");

    let output =
        make_temp_file(&dir, "target names", &content).expect("Couldn't create temp file B");

    TempEditingFiles { dir, input, output }
}

pub fn clean_temp_files(temp: TempEditingFiles) {
    temp.dir.close().expect("Couldn't close the temp dir");
}

pub fn ask_user_for_changes(temp: &TempEditingFiles) -> NextAction {
    start_editor_and_wait(&temp);

    let input_lines = read_paths_from_file(&temp.input);
    let output_lines = read_paths_from_file(&temp.output);

    match combine_files(input_lines, output_lines) {
        Ok(changes) => {
            let (changes, changes_count, error_count) = check_paths(changes);

            display_paths_changes(&changes);

            if error_count > 0 {
                println!(
                    "\n{}",
                    "There are errors in the paths renamings you requested."
                        .bold()
                        .red()
                );
                ask_user_to_retry()
            } else if changes_count == 0 {
                println!("\n{}", "You requested no path renaming.".bold().red());
                ask_user_to_retry()
            } else {
                ask_user_to_continue(clean_changes(changes), changes_count)
            }
        }
        Err(message) => {
            println!("{}\n", message.red());
            ask_user_to_retry()
        }
    }
}

/// Asks the user to retry editing or exit.
fn ask_user_to_retry() -> NextAction {
    println!("{}", "Do you want to retry editing?".red());

    let items = vec!["Edit", "Quit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(0);

    let answer = items[selection];

    println!("> {}", items[selection]);

    match answer {
        "Edit" => NextAction::Edit,
        _ => NextAction::Exit,
    }
}

/// Asks the user to continue with current changes, retry editing or exit.
fn ask_user_to_continue(changes: Vec<PathChange>, changes_count: u32) -> NextAction {
    println!(
        "\n{}",
        format!(
            "Do you confirm you want to rename {changes_count} path{}?",
            if changes_count > 1 { "s" } else { "" }
        )
        .bold()
        .green()
    );

    let items = vec!["Confirm", "Edit", "Quit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(0);

    let answer = items[selection];

    println!("> {}", items[selection]);

    match answer {
        "Confirm" => NextAction::Confirm(changes),
        "Edit" => NextAction::Edit,
        _ => NextAction::Exit,
    }
}

/// Creates a temporary file with a content
/// it will be returned (closed) with its path.
fn make_temp_file(
    parent: &TempDir,
    name: &str,
    content: &String,
) -> Result<PathBuf, std::io::Error> {
    let file_path = parent.path().join(name);
    let mut file = fs::File::create(&file_path)?;
    file.write(content.as_bytes())?;

    Ok(file_path)
}

/// Starts the editor and waits for it to be closed.
fn start_editor_and_wait(files: &TempEditingFiles) {
    let mut edit_cmd = Command::new("code");
    edit_cmd
        .arg("--wait")
        .arg("--diff")
        .arg(files.input.clone())
        .arg(files.output.clone());

    let mut child = edit_cmd
        .spawn()
        .expect("The editor process couldn't be spawned.");

    child
        .wait()
        .expect("An error occured waiting for the editor process.");
}

/// Read a list of paths from a file (or panics if it can't be read).
fn read_paths_from_file(path: &Path) -> Vec<PathBuf> {
    let file =
        fs::File::open(path).expect(format!("The file {:?} couldn't be read.", path).as_str());
    let buffer = std::io::BufReader::new(file);

    buffer
        .lines()
        .map(|line| {
            let line = line.expect("An error occured reading a line of the file.");
            PathBuf::from(line.trim())
        })
        .collect()
}

/// Verify that each path change is possible, or add a detailed error.
/// Also counts the blocking errors (only a unchanged path is counted as a non-blocking error)
fn check_paths(changes: Vec<PathChange>) -> (Vec<(PathChange, PathChangeResult)>, u32, u32) {
    let mut error_count = 0;
    let mut changes_count = 0;

    let results: Vec<(PathChange, PathChangeResult)> = changes
        .iter()
        .map(|change| {
            let result = check_path(&change, &changes);

            match result {
                Ok(_) => changes_count += 1,
                Err(PathChangeError::Unchanged) => (),
                Err(_) => error_count += 1,
            }

            ((change.0.clone(), change.1.clone()), result)
        })
        .collect();

    (results, changes_count, error_count)
}

/// Verify a path
fn check_path((input, output): &PathChange, parent_vec: &Vec<PathChange>) -> PathChangeResult {
    if input.eq(output) {
        Err(PathChangeError::Unchanged)
    } else if !input.exists() {
        Err(PathChangeError::InputDoesntExist)
    } else if input.to_str().unwrap_or("").is_empty() {
        Err(PathChangeError::InvalidInputPath)
    } else if output.exists() {
        Err(PathChangeError::OutputAlreadyExists)
    } else if output.to_str().unwrap_or("").is_empty() {
        Err(PathChangeError::InvalidOutputPath)
    } else if let Err(count) = check_uniqueness_of_output_path(output, &parent_vec) {
        Err(PathChangeError::SeveralOutputsAreTheSame(count))
    } else {
        Ok(())
    }
}

/// Verify that each path is unique in `paths`
fn check_uniqueness_of_output_path(
    output_path: &PathBuf,
    paths: &Vec<PathChange>,
) -> Result<(), u32> {
    let mut count = 0;

    for (_, output_path_2) in paths {
        if output_path.eq(output_path_2) {
            count += 1;
        }
    }

    if count > 1 {
        Err(count)
    } else {
        Ok(())
    }
}

fn combine_files(current: Vec<PathBuf>, target: Vec<PathBuf>) -> Result<Vec<PathChange>, String> {
    if current.len() != target.len() {
        return Err("The two files do not have the same number of lines.".to_string());
    }

    Ok(Vec::from_iter(current.into_iter().zip(target.into_iter())))
}

/// Filter and only keeps valid changes.
fn clean_changes(changes: Vec<(PathChange, PathChangeResult)>) -> Vec<PathChange> {
    changes
        .into_iter()
        .filter_map(|(change, result)| match result {
            Ok(_) => Some(change),
            _ => None,
        })
        .collect()
}

type TableRow = [ColoredString; 3];
type Table = Vec<TableRow>;

/// Displays a pretty list with all the transformations results
fn display_paths_changes(changes: &Vec<(PathChange, PathChangeResult)>) {
    let table = changes
        .iter()
        .filter_map(|(change, result)| -> Option<TableRow> {
            let input = change.0.to_str().unwrap_or("?");
            let output = change.1.to_str().unwrap_or("?");

            match &result {
                Ok(()) => Some([
                    input.bright_black().strikethrough(),
                    output.green(),
                    "Can be renamed".green(),
                ]),
                Err(PathChangeError::InputDoesntExist) => Some([
                    input.red(),
                    output.bright_black(),
                    "input doesn't exist".italic().red(),
                ]),
                Err(PathChangeError::InvalidInputPath) => Some([
                    input.red(),
                    output.bright_black(),
                    "invalid input path".italic().red(),
                ]),
                Err(PathChangeError::OutputAlreadyExists) => Some([
                    input.bright_black(),
                    output.red(),
                    "output already exists".italic().red(),
                ]),
                Err(PathChangeError::InvalidOutputPath) => Some([
                    input.bright_black(),
                    output.red(),
                    "invalid output path".italic().red(),
                ]),
                Err(PathChangeError::SeveralOutputsAreTheSame(n)) => Some([
                    input.bright_black(),
                    output.red(),
                    format!("several ({n}) outputs are the same").italic().red(),
                ]),

                // Unchanged lines are hidden
                Err(PathChangeError::Unchanged) => None,
            }
        })
        .collect();

    display_table(table, "→");
}

/// Displays a three columns table
pub fn display_table(table: Table, column_separator: &str) {
    println!("");

    let max_len_0 = get_max_length_of_column(&table, 0);
    let max_len_1 = get_max_length_of_column(&table, 1);

    for [s0, s1, s2] in table {
        // whitespaces
        let ws1 = " ".repeat(max_len_0 - s0.chars().count());
        let ws2 = " ".repeat(max_len_1 - s1.chars().count());

        println!("{s0}{ws1} {column_separator} {s1}{ws2} {column_separator} {s2}");
    }

    println!("");
}

/// Find the longuest string in one of the columns and returns its length.
fn get_max_length_of_column(table: &Table, column_id: usize) -> usize {
    table
        .iter()
        .map(|row| {
            row.get(column_id)
                .expect("Invalid column id")
                .chars()
                .count()
        })
        .reduce(|previous_len, current_len| max(previous_len, current_len))
        .unwrap_or(0)
}
