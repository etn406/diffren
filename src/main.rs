use std::path::PathBuf;
use clap::{Parser, Subcommand};

mod ren;
use ren::rename_command;

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
