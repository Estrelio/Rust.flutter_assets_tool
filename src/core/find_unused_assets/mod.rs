pub mod find_unused_assets {
    use std::path::PathBuf;
    use std::sync::Arc;

    use tokio::sync::{Mutex, RwLock};

    use crate::core::asset_metadata::asset_metadata::AssetMetadata;
    use crate::core::asset_metadata::parse_pubspec_yaml_assets;
    use crate::core::find_unused_assets::print_unused_assets_in_file_if_qualified::RemoveUnusedAssetsInStoreIfQualifiedError;
    use crate::core::flutter::get_flutter_project_inner_directory_path::get_flutter_project_lib_path;
    use crate::core::flutter::pubspec_yaml::read_pubspec_yaml_file;
    use crate::core::util::fs::read_file_recursively;
    use crate::core::util::fs::read_file_recursively::read_file_recursively;

    #[derive(Debug, thiserror::Error)]
    pub enum FindUnusedAssetsError {
        #[error("Error while reading file recursively: {0}")]
        ReadFileRecursivelyError(#[from] read_file_recursively::ReadFileRecursivelyError),
        #[error("Error while reading pubspec.yaml file: {0}")]
        ReadPubspecYamlFileError(
            #[from] crate::core::flutter::pubspec_yaml::ReadPubspecYamlFileError,
        ),
        #[error("Error while parsing pubspec.yaml assets: {0}")]
        ParsePubspecYamlAssetsError(
            #[from] crate::core::asset_metadata::ParsePubspecYamlAssetsError,
        ),
        #[error("Error while joining all tasks: {0}")]
        JoinAllError(#[from] tokio::task::JoinError),
        #[error("Error while removing unused assets in store if qualified: {0}")]
        RemoveUnusedAssetsInStoreIfQualifiedError(
            #[from] RemoveUnusedAssetsInStoreIfQualifiedError,
        ),
    }

    pub async fn find_unused_assets(
        flutter_project_path: &PathBuf,
        ignore_path_bufs: Vec<PathBuf>,
    ) -> Result<Vec<AssetMetadata>, FindUnusedAssetsError> {
        let pubspec_yaml = read_pubspec_yaml_file(&flutter_project_path)?;
        let asset_metadatum =
            parse_pubspec_yaml_assets(&flutter_project_path, &pubspec_yaml.flutter.assets)?;
        let asset_metadatum =
            filter_ignored_assets(&flutter_project_path, asset_metadatum, ignore_path_bufs);

        let asset_metadatum = Arc::new(RwLock::new(asset_metadatum));

        let join_handles: Arc<
            Mutex<
                Vec<tokio::task::JoinHandle<Result<(), RemoveUnusedAssetsInStoreIfQualifiedError>>>,
            >,
        > = Arc::new(Mutex::new(Vec::new()));

        let scanning_directory = get_flutter_project_lib_path(&flutter_project_path);

        read_file_recursively(
            &scanning_directory,
            Arc::new(|path: PathBuf| {
                let join_handles = join_handles.clone();
                let asset_metadatum = asset_metadatum.to_owned();
                let pubspec_yaml = pubspec_yaml.to_owned();
                async move {
                    if (&path).extension().unwrap_or_default() != "dart" {
                        return Ok(());
                    }

                    crate::core::find_unused_assets::print_unused_assets_in_file_if_qualified::remove_used_assets_in_store_if_qualified(
                        asset_metadatum.clone(),
                        join_handles.clone(),
                        &path,
                        &pubspec_yaml.flutter_gen.assets.outputs.style,
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

    fn filter_ignored_assets(
        flutter_project_path: &PathBuf,
        asset_metadatum: Vec<AssetMetadata>,
        ignore_path_bufs: Vec<PathBuf>,
    ) -> Vec<AssetMetadata> {
        let mut asset_metadatum_clone = asset_metadatum.clone();
        for ignore_path_buf in ignore_path_bufs {
            {
                let absolute_ignore_path_buf =
                    flutter_project_path.join(ignore_path_buf.to_owned());
                if absolute_ignore_path_buf.is_file() {
                    asset_metadatum_clone.retain(|asset_metadata| {
                        asset_metadata.get_asset_path() != &absolute_ignore_path_buf
                    });
                    continue;
                }
            }

            asset_metadatum_clone.retain(|asset_metadata| {
                asset_metadata.get_located_directory() != &ignore_path_buf
            });
        }

        asset_metadatum_clone
    }

    #[cfg(test)]
    mod tests {
        use crate::core::testing_util::artifact::get_test_artifact_directory;

        use super::*;

        #[test]
        fn filter_ignored_assets_works() {
            // Arrange
            let test_artifact_directory = get_test_artifact_directory().unwrap();
            let unique_id = uuid::Uuid::new_v4();
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
            let pubspec_yaml = read_pubspec_yaml_file(&flutter_project_path).unwrap();
            let asset_metadatum =
                parse_pubspec_yaml_assets(&flutter_project_path, &pubspec_yaml.flutter.assets)
                    .unwrap();

            // Act
            let result = filter_ignored_assets(
                &flutter_project_path,
                asset_metadatum,
                vec![
                    PathBuf::from("assets/fonts/"),
                    PathBuf::from("assets/images/coming_soon_ignored.png"),
                ],
            );

            // Assert
            let assets_image_directory = PathBuf::from("assets/images/");
            assert_eq!(
                result,
                vec![
                    AssetMetadata::new(
                        flutter_project_path
                            .join(&assets_image_directory)
                            .join("coming_soon_1.png"),
                        assets_image_directory.to_owned(),
                    ),
                    AssetMetadata::new(
                        flutter_project_path
                            .join(&assets_image_directory)
                            .join("coming_soon1.png"),
                        assets_image_directory.to_owned(),
                    ),
                ]
            );
        }
    }
}

mod print_unused_assets_in_file_if_qualified {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::sync::Arc;

    use tokio::sync::{Mutex, RwLock};

    use crate::core::asset_metadata::asset_metadata::AssetMetadata;
    use crate::core::asset_usage::{ASSET_USAGE_REGEX, ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME};
    use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputsStyle;

    #[derive(Debug, thiserror::Error)]
    pub enum RemoveUnusedAssetsInStoreIfQualifiedError {
        #[error("Error while reading file: {0}")]
        ReadFileError(std::io::Error),
    }

    pub async fn remove_used_assets_in_store_if_qualified(
        asset_metadatum: Arc<RwLock<Vec<AssetMetadata>>>,
        join_handles: Arc<
            Mutex<
                Vec<tokio::task::JoinHandle<Result<(), RemoveUnusedAssetsInStoreIfQualifiedError>>>,
            >,
        >,
        file_path: &PathBuf,
        flutter_gen_assets_outputs_style: &FlutterGenAssetsOutputsStyle,
    ) -> Result<(), RemoveUnusedAssetsInStoreIfQualifiedError> {
        let file_content = tokio::fs::read_to_string(&file_path)
            .await
            .map_err(|err| RemoveUnusedAssetsInStoreIfQualifiedError::ReadFileError(err))?;

        let handle = tokio::spawn(drop_used_assets(
            asset_metadatum,
            flutter_gen_assets_outputs_style.to_owned(),
            file_content,
        ));

        join_handles.lock().await.push(handle);

        Ok(())
    }

    async fn drop_used_assets(
        asset_metadatum: Arc<RwLock<Vec<AssetMetadata>>>,
        flutter_gen_assets_outputs_style: FlutterGenAssetsOutputsStyle,
        file_content: String,
    ) -> Result<(), RemoveUnusedAssetsInStoreIfQualifiedError> {
        let used_assets = ASSET_USAGE_REGEX
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
                        .compute_dart_usage_syntax(&flutter_gen_assets_outputs_style)
                        .unwrap()
                        != *used_asset
                });
            }
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
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

            // Act
            let result = drop_used_assets(
                asset_metadatum.clone(),
                flutter_gen_assets_outputs_style,
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

            // Act
            let result = drop_used_assets(
                asset_metadatum.clone(),
                flutter_gen_assets_outputs_style,
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
