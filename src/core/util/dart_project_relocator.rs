use std::path::PathBuf;

const DART_PROJECT_SENTINEL: &str = "pubspec.yaml";

#[derive(Debug, thiserror::Error)]
pub enum RelocateDartProjectError {
    #[error("Failed to check pubspec.yaml existence in {path}. {source}")]
    CheckPubspecYamlExistError {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Can't find Dart project in path")]
    DartProjectNotFound,
}

/// Relocate the dart project to ensure the path contains `pubspec.yaml` to be qualified as a Dart project.
pub fn relocate_dart_project(
    estimated_dart_project_path: PathBuf,
) -> Result<PathBuf, RelocateDartProjectError> {
    let dart_project_path = estimated_dart_project_path.join(DART_PROJECT_SENTINEL);
    if dart_project_path.try_exists().map_err(|err| {
        RelocateDartProjectError::CheckPubspecYamlExistError {
            source: err,
            path: dart_project_path.clone(),
        }
    })? {
        return Ok(dart_project_path
            // Remove the sentinel file name
            .parent()
            .unwrap()
            .to_path_buf());
    }

    relocate_dart_project(
        estimated_dart_project_path
            .parent()
            .ok_or_else(|| RelocateDartProjectError::DartProjectNotFound)?
            .to_path_buf(),
    )
}