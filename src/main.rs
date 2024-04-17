use std::path::PathBuf;

use clap::Parser;
use log::log;
use tokio::time::Instant;

use flutter_assets_tool::commands;
use flutter_assets_tool::commands::cli::{Cli, SubCommands};
use flutter_assets_tool::commands::migrate::MigrateCommand;
use flutter_assets_tool::core::util::dart_project_relocator::{relocate_dart_project, RelocateDartProjectError};
use flutter_assets_tool::logger::logger_setup;

#[tokio::main]
async fn main() {
    let stopwatch = Instant::now();
    match main_core().await {
        Ok(_) => {
            let duration = stopwatch.elapsed();
            log::info!(
                "Done. Took {duration} milliseconds.",
                duration = duration.as_millis()
            );
        }
        Err(err) => {
            log::error!(
                "{err}\n\n{chain}",
                err = err,
                chain = err
                    .chain()
                    .map(|err| err.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error("Failed to get current directory: {err}")]
    GetCurrentDirError {
        err: std::io::Error,
    },
    #[error("Failed to relocate dart project. {0}")]
    RelocateDartProjectError(#[from] RelocateDartProjectError),
    #[error("Dart project not found from path: {path}")]
    DartProjectNotFound {
        path: String,
    },
    // #[error("Failed to read pubspec.yaml file. {0}")]
    // ReadPubspecYamlFileError(#[from] expose_dart_class::core::dart::pubspec_yaml::ReadPubspecYamlFileError),
}

async fn main_core() {
    let cli = Cli::parse();

    logger_setup::setup_logger(cli.verbose);

    match cli.command {
        SubCommands::GenerateCompletions { shell } => {
            commands::generate_completions::generate_completions(shell);
        }
        SubCommands::Migrate { command } => {
            match command {
                MigrateCommand::AssetGen => {
                    
                }
            }
        }
        SubCommands::ListUnused { .. } => {}
    }
}


fn get_dart_project_path(dart_project_path: Option<String>) -> Result<PathBuf, MainError> {
    let dart_project_path = match dart_project_path.to_owned() {
        None => std::env::current_dir().map_err(|err| MainError::GetCurrentDirError { err })?,
        Some(dart_project_path) => {
            if dart_project_path == "." {
                std::env::current_dir().map_err(|err| MainError::GetCurrentDirError { err })?
            } else {
                dart_project_path.into()
            }
        }
    };
    let dart_project_path = relocate_dart_project(
        dart_project_path.clone(),
    )
        .map_err(|err| match err {
            RelocateDartProjectError::CheckPubspecYamlExistError { source, path } => {
                MainError::RelocateDartProjectError(RelocateDartProjectError::CheckPubspecYamlExistError { source, path })
            }
            RelocateDartProjectError::DartProjectNotFound => {
                MainError::DartProjectNotFound { path: dart_project_path.to_string_lossy().to_string() }
            }
        })?;
    Ok(dart_project_path)
}
