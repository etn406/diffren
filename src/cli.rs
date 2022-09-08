use clap::Parser;

use crate::config::TextEditor;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Path(s) of the files to list.
    /// Unix shell style patterns are supported.
    #[clap(value_parser)]
    pub paths: Vec<String>,

    #[clap(subcommand)]
    pub command: Option<Subcommand>,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    GetConfig,

    /// Displays the current command use to start
    UseEditor {
        #[clap(arg_enum, value_parser)]
        text_editor: TextEditor,
    },

    /// Set the custom editor's launch command, and use it.
    /// The strings "{current}" and "{target}" will be replaced
    /// by the paths of the temporary files to edit. "{current}"
    /// is optionnal since it doesn't need to be modified,
    /// but it is needed if you want a diff view between the two files.
    /// IE with VSCode: `diffren editor --set-custom="code --wait --diff {current} {target}"`
    SetCustomEditor {
        #[clap(value_parser)]
        command: String,
    },
}
