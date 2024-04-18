use clap::{Parser, Subcommand};
use crate::commands::generate_completions::Shell;
use crate::commands::migrate::MigrateCommand;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: SubCommands,

    /// Path to the flutter project.
    ///
    /// If not specified, the current directory will be used.
    #[clap(long, short = 'p')]
    pub flutter_project_path: Option<String>,

    /// Print verbose output.
    #[clap(long, short)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum SubCommands {
    /// Generate completions for your shell.
    GenerateCompletions {
        /// The shell to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Migrate your codebase.
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
    },
    /// List unused translations.
    ListUnused {
        /// Remove unused translations.
        #[clap(long, short, default_value = "false")]
        remove_unused: bool,

        /// Paths to ignore when searching for unused translations.
        ///
        /// If the specified path is a directory, ignore all files in the directory; otherwise,
        /// ignore the specified file.
        ///
        /// Make sure that you specify the directory path exactly how you specify it in your
        /// pubspec.yaml flutter.assets section.
        #[clap(long = "ignore-path")]
        ignore_paths: Option<Vec<String>>,
        
        #[clap(long = "exit-if-unused-exist", default_value = "false")]
        exit_if_unused_exist: bool,
    },
}
