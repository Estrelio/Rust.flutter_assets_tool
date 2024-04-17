use std::path::PathBuf;

use crate::core::asset_metadata::asset_metadata::AssetMetadata;

pub mod asset_metadata;

#[derive(Debug, thiserror::Error)]
pub enum ParsePubspecYamlAssetsError {
    #[error("Failed to read directory {path}. {source}")]
    ReadDirectoryError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Failed to fetch directory entry. {source}")]
    FetchDirectoryEntryError {
        source: std::io::Error,
    },
}

pub fn parse_pubspec_yaml_assets(
    flutter_project_path: &PathBuf,
    assets: &Vec<String>,
) -> Result<Vec<AssetMetadata>, ParsePubspecYamlAssetsError> {
    let mut asset_metadatum = Vec::new();
    for configured_asset in assets {
        let located_directory = flutter_project_path.join(&configured_asset);
        for dir_entry in (&located_directory).read_dir().map_err(
            |err| ParsePubspecYamlAssetsError::ReadDirectoryError {
                path: located_directory.to_owned(),
                source: err,
            },
        )? {
            let entry = dir_entry.map_err(
                |err| ParsePubspecYamlAssetsError::FetchDirectoryEntryError { source: err },
            )?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            asset_metadatum.push(AssetMetadata::new(
                path.to_owned(),
                PathBuf::from(configured_asset.to_owned()).to_owned(),
            ));
        };
    }

    Ok(asset_metadatum)
}

#[cfg(test)]
mod tests {
    use crate::core::flutter::pubspec_yaml;
    use crate::core::testing_util::artifact::get_test_artifact_directory;

    use super::*;

    #[test]
    fn parse_pubspec_yaml_assets_can_work() {
        // Arrange
        let test_artifact_directory = get_test_artifact_directory().unwrap();
        let unique_id = uuid::Uuid::new_v4();
        let test_artifact_directory = test_artifact_directory.join(unique_id.to_string());
        std::fs::create_dir_all(&test_artifact_directory).unwrap();
        fs_extra::dir::copy(
            std::env::current_dir().unwrap().join("tests/sample/flutter/dummy"),
            &test_artifact_directory,
            &fs_extra::dir::CopyOptions::new(),
        ).unwrap();
        let flutter_project_path = test_artifact_directory.join("dummy");
        let pubspec_yaml = pubspec_yaml::read_pubspec_yaml_file(&flutter_project_path).unwrap();

        // Act
        let result = parse_pubspec_yaml_assets(
            &flutter_project_path,
            &pubspec_yaml.flutter.assets,
        ).unwrap();

        // Assert
        let assets_image_directory = PathBuf::from("assets/images/");
        assert_eq!(
            result,
            vec![
                AssetMetadata::new(
                    flutter_project_path.join(&assets_image_directory).join("coming_soon_1.png"),
                    assets_image_directory.to_owned(),
                ),
                AssetMetadata::new(
                    flutter_project_path.join(&assets_image_directory).join("coming_soon1.png"),
                    assets_image_directory.to_owned(),
                ),
            ]
        );

        // Cleanup
        std::fs::remove_dir_all(&test_artifact_directory).unwrap();
    }
}
