use std::path::PathBuf;

use anyhow::Error;
use clap::Parser;
use tokio::time::Instant;

use flutter_assets_tool::commands;
use flutter_assets_tool::commands::cli::{Cli, SubCommands};
use flutter_assets_tool::commands::migrate::MigrateCommand;
use flutter_assets_tool::core::configuration;
use flutter_assets_tool::core::configuration::flutter_assets_tool::{
    FlutterAssetsTool, ReadFlutterAssetsToolFileError,
};
use flutter_assets_tool::core::util::dart_project_relocator::{
    relocate_dart_project, RelocateDartProjectError,
};
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

async fn main_core() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    logger_setup::setup_logger(cli.verbose);

    let flutter_project_path = get_flutter_project_path(cli.flutter_project_path)?;
    let flutter_assets_tool_file_result =
        configuration::flutter_assets_tool::read_flutter_assets_tool_file(&flutter_project_path);

    match cli.command {
        SubCommands::GenerateCompletions { shell } => {
            Ok(commands::generate_completions::generate_completions(shell))
        }
        SubCommands::Migrate { command } => match command {
            MigrateCommand::AssetGen => {
                commands::migrate::asset_gen::migrate::migrate_asset_gen_to_flutter_gen(
                    &flutter_project_path,
                )
                    .await?;

                log::info!("Migration completed.");

                Ok(())
            }
        },
        SubCommands::ListUnused {
            remove_unused,
            ignore_paths, exit_if_unused_exist,
        } => {
            let ignore_path_bufs =
                compute_ignore_path_bufs(flutter_assets_tool_file_result, ignore_paths)?;
            commands::list_unused::list_unused::list_unused(
                &flutter_project_path,
                remove_unused,
                ignore_path_bufs,
                exit_if_unused_exist,
            )
                .await?;

            Ok(())
        }
    }
}

fn compute_ignore_path_bufs(
    flutter_assets_tool_file_result: Result<FlutterAssetsTool, ReadFlutterAssetsToolFileError>,
    ignore_paths: Option<Vec<String>>,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    match ignore_paths {
        None => match flutter_assets_tool_file_result {
            Ok(flutter_assets_tool_file) => {
                let path_bufs: Vec<PathBuf> = flutter_assets_tool_file
                    .get_ignore_paths()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|path| PathBuf::from(path))
                    .collect();
                Ok(path_bufs)
            }
            Err(err) => match err {
                ReadFlutterAssetsToolFileError::OpenFileError { source } => {
                    log::debug!("Failed to open flutter_assets_tool.yaml file. {source}");
                    Ok(vec![])
                }
                ReadFlutterAssetsToolFileError::ParseFileError { source } => {
                    Err(Error::new(source))
                }
            },
        },
        Some(ignore_paths) => {
            return Ok(ignore_paths
                .into_iter()
                .map(|path| PathBuf::from(path))
                .collect());
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error("Failed to get current directory: {err}")]
    GetCurrentDirError { err: std::io::Error },
    #[error("Failed to relocate dart project. {0}")]
    RelocateDartProjectError(#[from] RelocateDartProjectError),
    #[error("Dart project not found from path: {path}")]
    DartProjectNotFound { path: String },
}

fn get_flutter_project_path(dart_project_path: Option<String>) -> Result<PathBuf, MainError> {
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
    let dart_project_path =
        relocate_dart_project(dart_project_path.clone()).map_err(|err| match err {
            RelocateDartProjectError::CheckPubspecYamlExistError { source, path } => {
                MainError::RelocateDartProjectError(
                    RelocateDartProjectError::CheckPubspecYamlExistError { source, path },
                )
            }
            RelocateDartProjectError::DartProjectNotFound => MainError::DartProjectNotFound {
                path: dart_project_path.to_string_lossy().to_string(),
            },
        })?;
    Ok(dart_project_path)
}
