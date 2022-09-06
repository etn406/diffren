use std::{cmp::max, io, path::PathBuf};

use colored::{ColoredString, Colorize};

use crate::validation::{Renaming, Validation, ValidationError};

type Table = Vec<TableRow>;
type TableRow = [ColoredString; 3];

/// Displays a pretty list with all the renamings validations and errors
pub fn display_validations(validations: &Vec<Validation>) {
    let table = validations
        .iter()
        .filter_map(|validation| -> Option<TableRow> {
            let fmt = |path: &PathBuf| path.to_str().unwrap_or("?").bright_black();

            match &validation {
                Ok(renaming) => Some([
                    fmt(&renaming.0).strikethrough(),
                    fmt(&renaming.1).green(),
                    "can be renamed".green(),
                ]),
                Err(ValidationError::FileDoesntExist(renaming)) => Some([
                    fmt(&renaming.0).red(),
                    fmt(&renaming.1),
                    "this file doesn't exist".italic().red(),
                ]),
                Err(ValidationError::InvalidCurrentPath(renaming)) => Some([
                    fmt(&renaming.0).red(),
                    fmt(&renaming.1),
                    "invalid current path".italic().red(),
                ]),
                Err(ValidationError::TargetAlreadyExists(renaming)) => Some([
                    fmt(&renaming.0),
                    fmt(&renaming.1).red(),
                    "output already exists".italic().red(),
                ]),
                Err(ValidationError::InvalidTargetPath(renaming)) => Some([
                    fmt(&renaming.0),
                    fmt(&renaming.1).red(),
                    "invalid target path".italic().red(),
                ]),
                Err(ValidationError::SeveralTargetsAreTheSame(renaming, count)) => Some([
                    fmt(&renaming.0),
                    fmt(&renaming.1).red(),
                    format!("several ({count}) outputs are the same")
                        .italic()
                        .red(),
                ]),

                // Unchanged lines aren't displayed
                Err(ValidationError::Unchanged(_)) => None,
            }
        })
        .collect();

    display_table(table, "→");
}

/// Display a table with all the results
pub fn display_results(results: &Vec<(Renaming, io::Result<()>)>) {
    let to_str = |p: &PathBuf| p.to_str().unwrap_or("?").normal();

    display_table(
        results
            .iter()
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
