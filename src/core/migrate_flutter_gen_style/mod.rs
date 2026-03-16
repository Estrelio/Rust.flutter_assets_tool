use std::path::{Path, PathBuf};
use std::sync::Arc;

use regex::Regex;
use tokio::sync::Mutex;

use crate::core::asset_metadata::AssetMetadata;
use crate::core::asset_metadata::parse_pubspec_yaml_assets;
use crate::core::asset_usage::AssetUsageRegexSetGroup;
use crate::core::flutter::get_flutter_project_inner_directory_path::get_flutter_project_lib_path;
use crate::core::flutter::pubspec_yaml::{
    FlutterGenAssetsOutputsStyle, PubspecYaml, read_pubspec_yaml_file,
};
use crate::core::util::fs::read_file_recursively;
use crate::core::util::fs::read_file_recursively::read_file_recursively;

#[derive(Debug, thiserror::Error)]
pub enum MigrateFlutterGenStyleError {
    #[error("Error while reading file recursively: {0}")]
    ReadFileRecursivelyError(#[from] read_file_recursively::ReadFileRecursivelyError),
    #[error("Error while reading pubspec.yaml file: {0}")]
    ReadPubspecYamlFileError(#[from] crate::core::flutter::pubspec_yaml::ReadPubspecYamlFileError),
    #[error("Error while parsing pubspec.yaml assets: {0}")]
    ParsePubspecYamlAssetsError(#[from] crate::core::asset_metadata::ParsePubspecYamlAssetsError),
    #[error("Error while joining all tasks: {0}")]
    JoinAllError(#[from] tokio::task::JoinError),
    #[error("Error while reading file: {0}")]
    ReadFileError(#[from] std::io::Error),
    #[error("Error while computing asset usage regex set group: {0}")]
    ComputeAssetUsageRegexSetError(
        #[from] crate::core::asset_usage::ComputeAssetUsageRegexSetError,
    ),
}

pub type MigrateFlutterGenStyleTaskJoinHandle =
    Arc<Mutex<Vec<tokio::task::JoinHandle<Result<(), MigrateFlutterGenStyleError>>>>>;

pub async fn migrate_flutter_gen_style(
    flutter_project_path: &Path,
    ignore_path_bufs: Vec<PathBuf>,
    previous_style: FlutterGenAssetsOutputsStyle,
) -> Result<(), MigrateFlutterGenStyleError> {
    let pubspec_yaml = read_pubspec_yaml_file(flutter_project_path)?;
    let asset_metadatum =
        parse_pubspec_yaml_assets(flutter_project_path, &pubspec_yaml.flutter.assets)?;
    let asset_metadatum = crate::core::filter_ignored_assets::filter_ignored_assets(
        flutter_project_path,
        asset_metadatum,
        ignore_path_bufs,
    );

    let join_handles: MigrateFlutterGenStyleTaskJoinHandle = Arc::new(Mutex::new(Vec::new()));

    let scanning_directory = get_flutter_project_lib_path(flutter_project_path);
    let asset_usage_regex_set_group =
        crate::core::asset_usage::compute_asset_usage_regex_set_group(
            &pubspec_yaml.flutter_gen.assets.outputs.class_name,
            &asset_metadatum,
            previous_style,
        )?;
    read_file_recursively(
        &scanning_directory,
        Arc::new(|path: PathBuf| {
            let join_handles = join_handles.clone();
            let pubspec_yaml = pubspec_yaml.to_owned();
            let asset_usage_regex_set_group = asset_usage_regex_set_group.to_owned();
            async move {
                if path.extension().unwrap_or_default() != "dart" {
                    return Ok(());
                }

                let handle = tokio::spawn(migrate_asset(
                    path,
                    pubspec_yaml,
                    asset_usage_regex_set_group,
                ));

                join_handles.lock().await.push(handle);

                Ok(())
            }
        }),
    )
    .await?;

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

    Ok(())
}

async fn migrate_asset(
    path: PathBuf,
    pubspec_yaml: PubspecYaml,
    asset_usage_regex_set_group: AssetUsageRegexSetGroup,
) -> Result<(), MigrateFlutterGenStyleError> {
    let mut file_content = tokio::fs::read_to_string(&path).await.map_err(|err| {
        read_file_recursively::ReadFileRecursivelyError::OtherError {
            source: Box::new(MigrateFlutterGenStyleError::ReadFileError(err)),
        }
    })?;

    for (regex, asset_metadata) in asset_usage_regex_set_group.regexes {
        file_content = migrate_asset_core(
            file_content,
            regex,
            asset_metadata,
            &pubspec_yaml.flutter_gen.assets.outputs.style,
            &pubspec_yaml.flutter_gen.assets.outputs.class_name,
        );
    }

    tokio::fs::write(&path, file_content).await.map_err(|err| {
        read_file_recursively::ReadFileRecursivelyError::OtherError {
            source: Box::new(MigrateFlutterGenStyleError::ReadFileError(err)),
        }
    })?;

    Ok(())
}

fn migrate_asset_core(
    file_content: String,
    regex: Regex,
    asset_metadata: AssetMetadata,
    to_style: &FlutterGenAssetsOutputsStyle,
    class_name: &str,
) -> String {
    regex
        .replace_all(&file_content, |_captures: &regex::Captures| -> String {
            let asset_usage_syntax = asset_metadata.compute_dart_usage_syntax(to_style).unwrap();
            format!(
                "{class_name}.{asset_usage_syntax}",
                class_name = class_name
            )
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::core::flutter::pubspec_yaml;
    use crate::core::testing_util::artifact::get_test_artifact_directory;

    use super::*;

    #[tokio::test]
    async fn migrate_asset_can_migrate_camel_case_when_the_target_is_snake_case() {
        // Arrange
        let test_artifact_directory = get_test_artifact_directory().unwrap();
        let unique_id = uuid::Uuid::now_v7();
        let test_artifact_directory = test_artifact_directory.join(unique_id.to_string());
        std::fs::create_dir_all(&test_artifact_directory).unwrap();
        fs_extra::dir::copy(
            std::env::current_dir()
                .unwrap()
                .join("tests/sample/flutter/snakeCaseDummy"),
            &test_artifact_directory,
            &fs_extra::dir::CopyOptions::new(),
        )
        .unwrap();
        let flutter_project_path = test_artifact_directory.join("snakeCaseDummy");
        let pubspec_yaml = pubspec_yaml::read_pubspec_yaml_file(&flutter_project_path).unwrap();
        let main_dart_file_path_buf = flutter_project_path.join("lib").join("main.dart");
        let asset_metadatum =
            parse_pubspec_yaml_assets(&flutter_project_path, &pubspec_yaml.flutter.assets).unwrap();
        let asset_usage_regex_set_group =
            crate::core::asset_usage::compute_asset_usage_regex_set_group(
                &pubspec_yaml.flutter_gen.assets.outputs.class_name,
                &asset_metadatum,
                FlutterGenAssetsOutputsStyle::CamelCase,
            )
            .unwrap();

        // Act
        migrate_asset(
            main_dart_file_path_buf.to_owned(),
            pubspec_yaml,
            asset_usage_regex_set_group,
        )
        .await
        .unwrap();

        // Assert
        assert_eq!(
            tokio::fs::read_to_string(main_dart_file_path_buf)
                .await
                .unwrap(),
            tokio::fs::read_to_string(
                flutter_project_path
                    .join("lib")
                    .join("expected_snake_case_main.dart"),
            )
            .await
            .unwrap(),
        );
    }

    #[test]
    fn migrate_asset_core_can_migrate_camel_case_when_target_is_dot_delimited() {
        // Arrange
        let file_content = r#"
            final List<String> images = [
  R.imagesComingSoon1.path;
  R.
  imagesComingSoon1.path;
            ];
        "#;
        let asset_metadata = AssetMetadata::new(
            PathBuf::from("assets/images/coming_soon_1.png"),
            PathBuf::from("assets/images"),
        );
        let asset_metadatum = vec![asset_metadata.to_owned()];
        let class_name = "R";
        let asset_usage_regex_set_group =
            crate::core::asset_usage::compute_asset_usage_regex_set_group(
                class_name,
                &asset_metadatum,
                FlutterGenAssetsOutputsStyle::CamelCase,
            )
            .unwrap();
        let regex = &asset_usage_regex_set_group
            .regexes
            .iter()
            .find(|x| x.1 == asset_metadata)
            .unwrap()
            .0;
        let to_style = FlutterGenAssetsOutputsStyle::DotDelimited;

        // Act
        let new_file_content = migrate_asset_core(
            file_content.to_string(),
            regex.to_owned(),
            asset_metadata,
            &to_style,
            class_name,
        );

        // Assert
        let expected_file_content = r#"
            final List<String> images = [
  R.images.comingSoon1.path;
  R.images.comingSoon1.path;
            ];
        "#;
        assert_eq!(new_file_content, expected_file_content,);
    }
}
