pub mod list_unused {
    use std::path::PathBuf;

    use crate::core::asset_metadata::asset_metadata::AssetMetadata;

    #[derive(Debug, thiserror::Error)]
    pub enum ListUnusedError {
        #[error("You have unused assets.")]
        UnusedAssetsExistError,
        #[error("Failed to find unused assets. {source}")]
        FindUnusedAssetsError {
            #[from]
            source: crate::core::find_unused_assets::find_unused_assets::FindUnusedAssetsError,
        },
        #[error("Failed to remove unused assets. {source}")]
        RemoveUnusedAssetsError {
            #[from]
            source: crate::commands::list_unused::remove_unused_asset::RemoveUnusedAssetsError,
        },
    }

    pub async fn list_unused(
        flutter_project_path: &PathBuf,
        remove_unused: bool,
        ignore_path_bufs: Vec<PathBuf>,
        exit_if_unused_exist: bool,
    ) -> Result<(), ListUnusedError> {
        let unused_assets: Vec<AssetMetadata> =
            crate::core::find_unused_assets::find_unused_assets::find_unused_assets(
                flutter_project_path,
                ignore_path_bufs,
            )
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
        }

        if !remove_unused {
            if exit_if_unused_exist {
                return Err(ListUnusedError::UnusedAssetsExistError);
            }
            
            return Ok(());
        }

        crate::commands::list_unused::remove_unused_asset::remove_unused_assets(unused_assets)
            .await?;
        log::info!("🧹Unused assets have been removed.");

        Ok(())
    }
}

mod remove_unused_asset {
    use crate::core::asset_metadata::asset_metadata::AssetMetadata;

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
