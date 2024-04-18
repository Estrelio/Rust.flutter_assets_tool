use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum ReadFileRecursivelyError {
    #[error("Failed to read directory in path: {path}. {source}")]
    ReadDirectoryError {
        path: PathBuf,
        source: tokio::io::Error,
    },
    #[error("Failed to fetch the next directory entry. {source}")]
    FetchNextDirectoryEntryError { source: tokio::io::Error },
    #[error("{source}")]
    OtherError {
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Read the file recursively.
///
/// When a file is encountered, the `f` function will be called.
#[async_recursion::async_recursion]
pub async fn read_file_recursively<F, Fut>(
    path: &PathBuf,
    f: Arc<F>,
) -> Result<(), ReadFileRecursivelyError>
where
    F: Fn(PathBuf) -> Fut + Send + Sync,
    Fut: Future<Output = Result<(), ReadFileRecursivelyError>> + Send + Sync,
{
    let mut dir_entries = tokio::fs::read_dir(&path).await.map_err(|err| {
        ReadFileRecursivelyError::ReadDirectoryError {
            path: path.to_owned(),
            source: err,
        }
    })?;

    while let Some(entry) = dir_entries
        .next_entry()
        .await
        .map_err(|err| ReadFileRecursivelyError::FetchNextDirectoryEntryError { source: err })?
    {
        let path = entry.path();

        log::debug!("Processing entity: {:?}", path);

        if path.is_dir() {
            read_file_recursively(&path, f.clone()).await?;
        } else {
            f(path).await?;
        }
    }
    Ok(())
}
