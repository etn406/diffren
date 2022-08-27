use colored::Colorize;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::process::Command;

use crate::display::display_validations;
use crate::filesystem::TempEditingFiles;
use crate::validation;
use crate::*;

/// Next action to perform :
pub enum NextAction {
    /// Confirm the changes
    Confirm(Vec<Renaming>),
    Edit,
    Exit,
}

/// Opens the editor, and after the user inputs their changes,
/// asks the user what to do next depending on the validation.
pub fn ask_user_for_changes(temp: &TempEditingFiles) -> NextAction {
    start_editor_and_wait(&temp);

    let current = filesystem::read_paths_from(&temp.current);
    let target = filesystem::read_paths_from(&temp.target);

    match validation::combine_paths_vecs(current, target) {
        Ok(renamings) => {
            let (renamings, changes_count, error_count) = validation::validate_renamings(renamings);

            display_validations(&renamings);

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
                ask_user_to_continue(validation::keep_valid_renamings(renamings), changes_count)
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
fn ask_user_to_continue(changes: Vec<Renaming>, changes_count: u32) -> NextAction {
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

/// Starts the editor and waits for it to be closed.
fn start_editor_and_wait(files: &TempEditingFiles) {
    let mut edit_cmd = Command::new("code");
    edit_cmd
        .arg("--wait")
        .arg("--diff")
        .arg(files.current.clone())
        .arg(files.target.clone());

    let mut child = edit_cmd
        .spawn()
        .expect("The editor process couldn't be spawned.");

    child
        .wait()
        .expect("An error occured waiting for the editor process.");
}
