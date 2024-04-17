use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum GetTestArtifactDirectoryError {
    #[error("Failed to get current directory. {inner_error}")]
    GetCurrentDirectoryError { inner_error: std::io::Error },
}

/// Get the test artifact directory.
///
/// Create the directory if it does not exist.
pub fn get_test_artifact_directory() -> Result<PathBuf, GetTestArtifactDirectoryError> {
    let artifacts_dir = std::env::current_dir()
        .map_err(
            |err| GetTestArtifactDirectoryError::GetCurrentDirectoryError { inner_error: err },
        )?
        // Place the artifact in the target directory. Placing it in the current directory is too
        // dangerous.
        .join("target")
        .join("artifacts");
    std::fs::create_dir_all(&artifacts_dir).unwrap();
    Ok(artifacts_dir)
}
