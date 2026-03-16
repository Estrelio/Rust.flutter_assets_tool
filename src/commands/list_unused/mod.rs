use std::path::{Path, PathBuf};

use crate::core::asset_metadata::AssetMetadata;

#[derive(Debug, thiserror::Error)]
pub enum ListUnusedError {
    #[error("You have unused assets. \nYou can run flutter_assets_tool list-unused --remove-unused to remove them.")]
    UnusedAssetsExistError,
    #[error("Failed to find unused assets. {source}")]
    FindUnusedAssetsError {
        #[from]
        source: crate::core::find_unused_assets::FindUnusedAssetsError,
    },
    #[error("Failed to remove unused assets. {source}")]
    RemoveUnusedAssetsError {
        #[from]
        source: crate::commands::list_unused::remove_unused_asset::RemoveUnusedAssetsError,
    },
}

pub async fn list_unused(
    flutter_project_path: &Path,
    remove_unused: bool,
    ignore_path_bufs: Vec<PathBuf>,
    exit_if_unused_exist: bool,
) -> Result<(), ListUnusedError> {
    let unused_assets: Vec<AssetMetadata> =
        crate::core::find_unused_assets::find_unused_assets(flutter_project_path, ignore_path_bufs)
            .await?;

    if !unused_assets.is_empty() {
        log::info!("Unused assets:");
        log::info!("---------------------------------------------------");
        for asset_metadata in &unused_assets {
            log::info!(
                "🗑️ {}",
                asset_metadata
                    .get_asset_path()
                    .to_string_lossy()
                    .to_string()
            );
        }
        log::info!("---------------------------------------------------");
        if exit_if_unused_exist {
            return Err(ListUnusedError::UnusedAssetsExistError);
        }
    }

    if !remove_unused {
        return Ok(());
    }

    crate::commands::list_unused::remove_unused_asset::remove_unused_assets(unused_assets).await?;
    log::info!("🧹Unused assets have been removed.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::testing_util::artifact::get_test_artifact_directory;

    use super::*;

    #[tokio::test]
    async fn list_unused_return_unused_assets_exist_error_when_requested() {
        // Arrange
        let test_artifact_directory = get_test_artifact_directory().unwrap();
        let unique_id = uuid::Uuid::now_v7();
        let test_artifact_directory = test_artifact_directory.join(unique_id.to_string());
        std::fs::create_dir_all(&test_artifact_directory).unwrap();
        fs_extra::dir::copy(
            std::env::current_dir()
                .unwrap()
                .join("tests/sample/flutter/dummy"),
            &test_artifact_directory,
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();
        let flutter_project_path = test_artifact_directory.join("dummy");
        let ignore_path_bufs: Vec<PathBuf> = Vec::new();
        let exit_if_unused_exist = true;

        // Act
        let result = list_unused(
            &flutter_project_path,
            false,
            ignore_path_bufs,
            exit_if_unused_exist,
        )
        .await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ListUnusedError::UnusedAssetsExistError => {
                assert!(true)
            }
            ListUnusedError::FindUnusedAssetsError { .. }
            | ListUnusedError::RemoveUnusedAssetsError { .. } => {
                assert!(false)
            }
        }
    }
}

mod remove_unused_asset {
    use crate::core::asset_metadata::AssetMetadata;

    #[derive(Debug, thiserror::Error)]
    pub enum RemoveUnusedAssetsError {
        #[error("Failed to remove unused assets. {source}")]
        RemoveUnusedAssetsError {
            #[from]
            source: fs_extra::error::Error,
        },
    }

    pub async fn remove_unused_assets(
        unused_assets: Vec<AssetMetadata>,
    ) -> Result<(), RemoveUnusedAssetsError> {
        fs_extra::remove_items(&unused_assets)?;

        Ok(())
    }
}
