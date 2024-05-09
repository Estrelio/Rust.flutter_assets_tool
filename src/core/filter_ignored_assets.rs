use std::path::{Path, PathBuf};

use crate::core::asset_metadata::AssetMetadata;

pub fn filter_ignored_assets(
    flutter_project_path: &Path,
    asset_metadatum: Vec<AssetMetadata>,
    ignore_path_bufs: Vec<PathBuf>,
) -> Vec<AssetMetadata> {
    let mut asset_metadatum_clone = asset_metadatum.clone();
    for ignore_path_buf in ignore_path_bufs {
        {
            let absolute_ignore_path_buf = flutter_project_path.join(&ignore_path_buf);
            if absolute_ignore_path_buf.is_file() {
                asset_metadatum_clone.retain(|asset_metadata| {
                    asset_metadata.get_asset_path() != &absolute_ignore_path_buf
                });
                continue;
            }
        }

        asset_metadatum_clone
            .retain(|asset_metadata| asset_metadata.get_located_directory() != &ignore_path_buf);
    }

    asset_metadatum_clone
}

#[cfg(test)]
mod tests {
    use crate::core::asset_metadata::parse_pubspec_yaml_assets;
    use crate::core::flutter::pubspec_yaml::read_pubspec_yaml_file;
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
            parse_pubspec_yaml_assets(&flutter_project_path, &pubspec_yaml.flutter.assets).unwrap();

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
