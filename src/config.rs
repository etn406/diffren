use crate::filesystem::TempEditingFiles;
use colored::Colorize;
use preferences::{AppInfo, Preferences};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, process::Command};

const KEY: &str = "user_prefs";

const APP_INFO: AppInfo = AppInfo {
    qualifier: "",
    organization: "etn406",
    application: "diffren",
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct DiffrenConfig {
    pub custom_editor: String,
    pub editor_to_use: Option<TextEditor>,
}

/// Available text editors
#[derive(
    Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug, Serialize, Deserialize,
)]
pub enum TextEditor {
    Vscode,
    Vscodium,
    Custom,
}

impl Display for TextEditor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            match *self {
                TextEditor::Vscode => "VSCode",
                TextEditor::Vscodium => "VSCodium",
                TextEditor::Custom => "custom editor",
            }
        )
    }
}

/// Reads and returns the current configuration.
pub fn read_config() -> Result<DiffrenConfig, String> {
    match DiffrenConfig::load(&APP_INFO, KEY) {
        Ok(config) => Ok(config),
        Err(err) => Err(format!("Couldn't read configuration: {}", err)),
    }
}

/// Save the custom editor launch command.
pub fn set_custom_editor_command(command: &String) -> Result<(), String> {
    if !command.contains("{target}") {
        return Err("The command doesn't contain `{target}`.".to_string());
    }

    let mut config = read_config().unwrap_or_default();
    config.custom_editor = command.to_owned();

    match save_config(&config) {
        Ok(()) => {
            println!("The custom editor launch command has been set with:");
            println!("  > {}", config.custom_editor.bright_black());
            Ok(())
        }
        Err(err) => Err(err),
    }
}

/// Save the editor to use.
pub fn set_editor_to_use(editor: &TextEditor) -> Result<(), String> {
    let mut config = read_config().unwrap_or_default();
    config.editor_to_use = Some(*editor);

    match save_config(&config) {
        Ok(()) => {
            println!(
                "The editor to use is now: {}",
                editor.to_string().bright_black()
            );
            Ok(())
        }
        Err(err) => Err(err),
    }
}

// Save the configuration.
fn save_config(config: &DiffrenConfig) -> Result<(), String> {
    match config.save(&APP_INFO, KEY) {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("Couldn't save configuration: {}", err)),
    }
}

/// Returns a `Command` ready to be spawned with the editor set in the configuration.
pub fn get_editor_command(files: &TempEditingFiles) -> Command {
    let mut cmd;

    match read_config().unwrap().editor_to_use.unwrap() {
        TextEditor::Vscode => {
            cmd = Command::new("code");
            cmd.arg("--wait")
                .arg("--diff")
                .arg(&files.current)
                .arg(&files.target);
        }
        TextEditor::Vscodium => {
            cmd = Command::new("codium");
            cmd.arg("--wait")
                .arg("--diff")
                .arg(&files.current)
                .arg(&files.target);
        }
        TextEditor::Custom => todo!(),
    }

    cmd
}

/// Verify that an editor is defined
pub fn is_there_an_editor_to_use() -> Result<(), String> {
    let config = read_config().unwrap_or_default();

    match config.editor_to_use {
        Some(TextEditor::Custom) => {
            if config.custom_editor == "" {
                Err("No custom editor launch command defined".to_string())
            } else {
                Ok(())
            }
        }
        Some(_) => Ok(()),
        None => Err("You have to set which text editor to use!"
            .bold()
            .to_string()),
    }
}

pub fn get_path_to_config() -> String {
    preferences::prefs_base_dir(&APP_INFO)
        .unwrap_or(PathBuf::new())
        .to_str()
        .unwrap_or("?")
        .to_string()
}

pub fn print_config() -> Result<(), String> {
    match read_config() {
        Ok(config) => {
            println!(
                "{} from {}",
                "Current configuration".bold(),
                get_path_to_config().italic()
            );

            if config.custom_editor.is_empty() {
                println!("• No custom editor launch command defined.");
            } else {
                println!("• Custom editor launch command:",);
                println!("  > {}", config.custom_editor.bright_black());
            }

            if let Some(editor) = config.editor_to_use {
                println!("• Editor to use: {}", editor.to_string().bright_black());
            } else {
                println!("• Editor to use: {}", "undefined".red());
                println!(
                    "{}",
                    "You have to set which text editor to use!".bold().red()
                );
            }

            Ok(())
        }
        Err(err) => Err(err),
    }
}
