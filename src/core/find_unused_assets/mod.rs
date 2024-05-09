use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::core::asset_metadata::AssetMetadata;
use crate::core::asset_metadata::parse_pubspec_yaml_assets;
use crate::core::asset_usage::get_asset_usage_regex;
use crate::core::find_unused_assets::print_unused_assets_in_file_if_qualified::{
    RemoveUnusedAssetsInStoreIfQualifiedError, RemoveUsedAssetsInStoreIfQualifiedTaskJoinHandle,
};
use crate::core::flutter::get_flutter_project_inner_directory_path::get_flutter_project_lib_path;
use crate::core::flutter::pubspec_yaml::read_pubspec_yaml_file;
use crate::core::util::fs::read_file_recursively;
use crate::core::util::fs::read_file_recursively::read_file_recursively;

#[derive(Debug, thiserror::Error)]
pub enum FindUnusedAssetsError {
    #[error("Error while reading file recursively: {0}")]
    ReadFileRecursivelyError(#[from] read_file_recursively::ReadFileRecursivelyError),
    #[error("Error while reading pubspec.yaml file: {0}")]
    ReadPubspecYamlFileError(#[from] crate::core::flutter::pubspec_yaml::ReadPubspecYamlFileError),
    #[error("Error while parsing pubspec.yaml assets: {0}")]
    ParsePubspecYamlAssetsError(#[from] crate::core::asset_metadata::ParsePubspecYamlAssetsError),
    #[error("Error while joining all tasks: {0}")]
    JoinAllError(#[from] tokio::task::JoinError),
    #[error("Error while removing unused assets in store if qualified: {0}")]
    RemoveUnusedAssetsInStoreIfQualifiedError(#[from] RemoveUnusedAssetsInStoreIfQualifiedError),
}

pub async fn find_unused_assets(
    flutter_project_path: &Path,
    ignore_path_bufs: Vec<PathBuf>,
) -> Result<Vec<AssetMetadata>, FindUnusedAssetsError> {
    let pubspec_yaml = read_pubspec_yaml_file(flutter_project_path)?;
    let asset_metadatum =
        parse_pubspec_yaml_assets(flutter_project_path, &pubspec_yaml.flutter.assets)?;
    let asset_metadatum = crate::core::filter_ignored_assets::filter_ignored_assets(
        flutter_project_path,
        asset_metadatum,
        ignore_path_bufs,
    );

    let asset_metadatum = Arc::new(RwLock::new(asset_metadatum));

    let join_handles: RemoveUsedAssetsInStoreIfQualifiedTaskJoinHandle =
        Arc::new(Mutex::new(Vec::new()));

    let scanning_directory = get_flutter_project_lib_path(flutter_project_path);

    let asset_usage_regex =
        get_asset_usage_regex(&pubspec_yaml.flutter_gen.assets.outputs.class_name);
    read_file_recursively(
        &scanning_directory,
        Arc::new(|path: PathBuf| {
            let join_handles = join_handles.clone();
            let asset_metadatum = asset_metadatum.to_owned();
            let pubspec_yaml = pubspec_yaml.to_owned();
            let asset_usage_regex = asset_usage_regex.to_owned();
            async move {
                if path.extension().unwrap_or_default() != "dart" {
                    return Ok(());
                }

                crate::core::find_unused_assets::print_unused_assets_in_file_if_qualified::remove_used_assets_in_store_if_qualified(
                    &asset_usage_regex,
                    asset_metadatum.clone(),
                    join_handles.clone(),
                    &path,
                    &pubspec_yaml.flutter_gen.assets.outputs,
                ).await.map_err(
                    |err| read_file_recursively::ReadFileRecursivelyError::OtherError {
                        source: Box::new(err),
                    },
                )?;
                Ok(())
            }
        }),
    ).await?;

    {
        let mut handles = join_handles.lock().await;
        // Ensure that the vector of join handles can be used without keeping the mutex locked,
        // which would block other tasks from pushing their join handles into the vector.
        let handles = std::mem::take(&mut *handles);
        let results = futures_util::future::join_all(handles).await;
        for result in results {
            result??
        }
    }

    let asset_metadatum = asset_metadatum.read().await.to_owned();

    Ok(asset_metadatum)
}

mod print_unused_assets_in_file_if_qualified {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::sync::Arc;

    use tokio::sync::{Mutex, RwLock};

    use crate::core::asset_metadata::AssetMetadata;
    use crate::core::asset_usage::ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME;
    use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputs;

    #[derive(Debug, thiserror::Error)]
    pub enum RemoveUnusedAssetsInStoreIfQualifiedError {
        #[error("Error while reading file: {0}")]
        ReadFileError(std::io::Error),
    }

    pub type RemoveUsedAssetsInStoreIfQualifiedTaskJoinHandle = Arc<
        Mutex<Vec<tokio::task::JoinHandle<Result<(), RemoveUnusedAssetsInStoreIfQualifiedError>>>>,
    >;

    pub async fn remove_used_assets_in_store_if_qualified(
        asset_usage_regex: &regex::Regex,
        asset_metadatum: Arc<RwLock<Vec<AssetMetadata>>>,
        join_handles: RemoveUsedAssetsInStoreIfQualifiedTaskJoinHandle,
        file_path: &PathBuf,
        flutter_gen_assets_outputs: &FlutterGenAssetsOutputs,
    ) -> Result<(), RemoveUnusedAssetsInStoreIfQualifiedError> {
        let file_content = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(RemoveUnusedAssetsInStoreIfQualifiedError::ReadFileError)?;

        let handle = tokio::spawn(drop_used_assets(
            asset_usage_regex.to_owned(),
            asset_metadatum,
            flutter_gen_assets_outputs.to_owned(),
            file_content,
        ));

        join_handles.lock().await.push(handle);

        Ok(())
    }

    async fn drop_used_assets(
        asset_usage_regex: regex::Regex,
        asset_metadatum: Arc<RwLock<Vec<AssetMetadata>>>,
        flutter_gen_assets_outputs: FlutterGenAssetsOutputs,
        file_content: String,
    ) -> Result<(), RemoveUnusedAssetsInStoreIfQualifiedError> {
        let used_assets = asset_usage_regex
            .captures_iter(&file_content)
            .map(|capture| {
                capture
                    .name(ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME)
                    .unwrap()
                    .as_str()
                    .to_owned()
            })
            .collect::<HashSet<String>>();

        {
            let mut write_guard = asset_metadatum.write().await;
            for used_asset in used_assets {
                write_guard.retain(|asset_metadata| {
                    asset_metadata
                        .compute_dart_usage_syntax(&flutter_gen_assets_outputs.style)
                        .unwrap()
                        != *used_asset
                });
            }
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::core::asset_usage::get_asset_usage_regex;
        use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputsStyle;

        use super::*;

        #[tokio::test]
        async fn drop_used_assets_is_empty_when_all_the_assets_have_usage() {
            // Arrange
            let asset_metadata_1 = AssetMetadata::new(
                PathBuf::from("assets/images/image_1.png"),
                PathBuf::from("assets/images"),
            );
            let asset_metadata_2 = AssetMetadata::new(
                PathBuf::from("assets/images/image_2.png"),
                PathBuf::from("assets/images"),
            );
            let asset_metadata_3 = AssetMetadata::new(
                PathBuf::from("assets/images/image_3.png"),
                PathBuf::from("assets/images"),
            );
            let asset_metadatum = vec![
                asset_metadata_1.to_owned(),
                asset_metadata_2.to_owned(),
                asset_metadata_3.to_owned(),
            ];
            let asset_metadatum = Arc::new(RwLock::new(asset_metadatum));
            let flutter_gen_assets_outputs_style = FlutterGenAssetsOutputsStyle::SnakeCase;
            let file_content = format!(
                "
                final List<String> images = [
                    R.{asset_1}.path,
                    R
                        .{asset_2}.path,
                    R.
                        {asset_3}.path,
                ];
            ",
                asset_1 = asset_metadata_1
                    .compute_dart_usage_syntax(&flutter_gen_assets_outputs_style)
                    .unwrap(),
                asset_2 = asset_metadata_2
                    .compute_dart_usage_syntax(&flutter_gen_assets_outputs_style)
                    .unwrap(),
                asset_3 = asset_metadata_3
                    .compute_dart_usage_syntax(&flutter_gen_assets_outputs_style)
                    .unwrap(),
            )
            .to_string();
            let asset_usage_regex = get_asset_usage_regex("R");

            // Act
            let result = drop_used_assets(
                asset_usage_regex,
                asset_metadatum.clone(),
                FlutterGenAssetsOutputs {
                    class_name: "R".to_string(),
                    style: flutter_gen_assets_outputs_style,
                },
                file_content,
            )
            .await;

            // Assert
            assert!(result.is_ok());
            let read_guard = asset_metadatum.read().await;
            assert_eq!(read_guard.len(), 0);
        }

        #[tokio::test]
        async fn drop_used_assets_retain_unused_asset_when_one_asset_has_no_usage() {
            // Arrange
            let asset_metadata_1 = AssetMetadata::new(
                PathBuf::from("assets/images/image_1.png"),
                PathBuf::from("assets/images"),
            );
            let asset_metadata_2 = AssetMetadata::new(
                PathBuf::from("assets/images/image_2.png"),
                PathBuf::from("assets/images"),
            );
            let asset_metadata_3 = AssetMetadata::new(
                PathBuf::from("assets/images/image_3.png"),
                PathBuf::from("assets/images"),
            );
            let asset_metadatum = vec![
                asset_metadata_1.to_owned(),
                asset_metadata_2.to_owned(),
                asset_metadata_3.to_owned(),
            ];
            let asset_metadatum = Arc::new(RwLock::new(asset_metadatum));
            let flutter_gen_assets_outputs_style = FlutterGenAssetsOutputsStyle::SnakeCase;
            let file_content = format!(
                "
                final List<String> images = [
                    R.{asset_1}.path,
                    R
                        .{asset_2}.path,
                ];
            ",
                asset_1 = asset_metadata_1
                    .compute_dart_usage_syntax(&flutter_gen_assets_outputs_style)
                    .unwrap(),
                asset_2 = asset_metadata_2
                    .compute_dart_usage_syntax(&flutter_gen_assets_outputs_style)
                    .unwrap(),
            )
            .to_string();
            let asset_usage_regex = get_asset_usage_regex("R");

            // Act
            let result = drop_used_assets(
                asset_usage_regex,
                asset_metadatum.clone(),
                FlutterGenAssetsOutputs {
                    class_name: "R".to_string(),
                    style: flutter_gen_assets_outputs_style,
                },
                file_content,
            )
            .await;

            // Assert
            assert!(result.is_ok());
            let read_guard = asset_metadatum.read().await;
            assert_eq!(read_guard.len(), 1);
            assert_eq!(
                read_guard[0].get_asset_path(),
                asset_metadata_3.get_asset_path()
            );
        }
    }
}
