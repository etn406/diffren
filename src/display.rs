use std::{cmp::max, io, path::PathBuf};

use colored::{ColoredString, Colorize};

use crate::validation::{Renaming, Validation, ValidationError};

type Table = Vec<TableRow>;
type TableRow = [ColoredString; 3];

/// Displays a pretty list with all the renamings validations and errors
pub fn display_validations(renamings: &Vec<(Renaming, Validation)>) {
    let table = renamings
        .iter()
        .filter_map(|(renaming, validation)| -> Option<TableRow> {
            let current = renaming.0.to_str().unwrap_or("?");
            let target = renaming.1.to_str().unwrap_or("?");

            match &validation {
                Ok(()) => Some([
                    current.bright_black().strikethrough(),
                    target.green(),
                    "can be renamed".green(),
                ]),
                Err(ValidationError::FileDoesntExist) => Some([
                    current.red(),
                    target.bright_black(),
                    "this file doesn't exist".italic().red(),
                ]),
                Err(ValidationError::InvalidCurrentPath) => Some([
                    current.red(),
                    target.bright_black(),
                    "invalid current path".italic().red(),
                ]),
                Err(ValidationError::TargetAlreadyExists) => Some([
                    current.bright_black(),
                    target.red(),
                    "output already exists".italic().red(),
                ]),
                Err(ValidationError::InvalidTargetPath) => Some([
                    current.bright_black(),
                    target.red(),
                    "invalid target path".italic().red(),
                ]),
                Err(ValidationError::SeveralTargetsAreTheSame(n)) => Some([
                    current.bright_black(),
                    target.red(),
                    format!("several ({n}) outputs are the same").italic().red(),
                ]),

                // Unchanged lines are hidden
                Err(ValidationError::Unchanged) => None,
            }
        })
        .collect();

    display_table(table, "→");
}

/// Display a table with all the results
pub fn display_results(results: Vec<(Renaming, io::Result<()>)>) {
    let to_str = |p: PathBuf| p.to_str().unwrap_or("?").normal();

    display_table(
        results
            .into_iter()
            .map(|((current, target), result)| match result {
                Ok(()) => [to_str(current), to_str(target), "✓ Renamed".green()],
                Err(e) => [to_str(current), to_str(target), format!("✗ {}", e).red()],
            })
            .collect(),
        "→",
    );
}

/// Displays a three columns table
fn display_table(table: Table, column_separator: &str) {
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
