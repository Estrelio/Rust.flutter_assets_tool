use std::path::PathBuf;
use std::sync::Arc;
use crate::core::flutter::pubspec_yaml::{PubspecYaml, read_pubspec_yaml_file, ReadPubspecYamlFileError};
use crate::core::util::fs::read_file_recursively;

#[derive(Debug, thiserror::Error)]
pub enum MigrateAssetGenToFlutterGenError {
    #[error("{0}")]
    ReadPubspecYamlFileError(#[from] ReadPubspecYamlFileError),
    #[error("{0}")]
    ReadFileRecursivelyError(#[from] read_file_recursively::ReadFileRecursivelyError),
}
pub async fn migrate_asset_gen_to_flutter_gen(
    flutter_project_path: &PathBuf,
) -> Result<(), MigrateAssetGenToFlutterGenError>{
    let pubspec_yaml: PubspecYaml = read_pubspec_yaml_file(flutter_project_path)?;
    let scanning_directory = flutter_project_path
        // Dart project picks up the lib directory for compilation. Start scanning here for better
        // performance.
        .join("lib");
    read_file_recursively::read_file_recursively(
        scanning_directory,
        Arc::new(|path| async move {

            Ok(())
        }),
    ).await?;
    Ok(())
}
