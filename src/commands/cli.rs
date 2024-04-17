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
    },
}
