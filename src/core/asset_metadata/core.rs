use std::path::{Path, PathBuf};

use recase::ReCase;

use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputsStyle;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssetMetadata {
    /// The path to the asset.
    asset_path: PathBuf,
    /// The exact configuration from the `pubspec.yaml` file.
    located_directory: PathBuf,
}

impl AsRef<Path> for AssetMetadata {
    fn as_ref(&self) -> &Path {
        self.get_asset_path()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssetMetadataError {
    #[error("Failed to retrieve the file name from {path}.")]
    RetrieveFileStemError { path: PathBuf },
}

impl AssetMetadata {
    pub fn new(asset_path: PathBuf, located_directory: PathBuf) -> Self {
        Self {
            asset_path,
            located_directory,
        }
    }

    pub fn get_located_directory(&self) -> &PathBuf {
        &self.located_directory
    }

    /// Get the absolute path to the asset.
    pub fn get_asset_path(&self) -> &PathBuf {
        &self.asset_path
    }

    /// Compute a String of the possible usage of the asset in the Dart project.
    pub fn compute_dart_usage_syntax(
        &self,
        flutter_gen_assets_outputs_style: &FlutterGenAssetsOutputsStyle,
    ) -> Result<String, AssetMetadataError> {
        let file_stem = self
            .asset_path
            .file_stem()
            .ok_or_else(|| AssetMetadataError::RetrieveFileStemError {
                path: self.asset_path.clone(),
            })?
            .to_string_lossy()
            .to_string();

        let file_stem_recase = ReCase::new(&file_stem);

        // We need to separate the directory name from the file name because ReCase
        // considers number as a separate word.
        let relative_asset_path: PathBuf = self
            .located_directory
            .components()
            // `flutter_gen` doesn't support placing assets outside the Flutter recommended
            // `assets` directory. Plus, `flutter_gen` doesn't include the `assets`
            // directory in its generated code.
            .skip(1)
            .collect();
        match flutter_gen_assets_outputs_style {
            FlutterGenAssetsOutputsStyle::SnakeCase => {
                let mut result = String::new();

                let relative_asset_path = relative_asset_path.to_string_lossy().to_string();
                if !relative_asset_path.is_empty() {
                    result.push_str(&ReCase::new(&relative_asset_path).snake_case());
                    result.push('_');
                }
                result.push_str(&file_stem_recase.snake_case());
                Ok(result)
            }
            FlutterGenAssetsOutputsStyle::CamelCase => {
                let relative_asset_path = relative_asset_path.to_string_lossy().to_string();
                if !relative_asset_path.is_empty() {
                    let mut result = String::new();
                    result.push_str(&ReCase::new(&relative_asset_path).camel_case());
                    result.push_str(&file_stem_recase.pascal_case());
                    Ok(result)
                } else {
                    Ok(file_stem_recase.camel_case())
                }
            }
            FlutterGenAssetsOutputsStyle::DotDelimited => {
                let mut result = String::new();

                let relative_asset_path = relative_asset_path.to_string_lossy().to_string();
                if !relative_asset_path.is_empty() {
                    result.push_str(&ReCase::new(&relative_asset_path).dot_case());
                    result.push('.');
                }
                result.push_str(&file_stem_recase.camel_case());
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_dart_usage_syntax_passes_when_requesting_snake_case() {
        // Arrange
        let located_directory = PathBuf::from("assets/images");
        let asset_path = located_directory.join("image_1.png");
        let asset_metadata = AssetMetadata::new(asset_path, located_directory);

        // Act
        let usage_syntax = asset_metadata
            .compute_dart_usage_syntax(&FlutterGenAssetsOutputsStyle::SnakeCase)
            .unwrap();

        // Assert
        assert_eq!(usage_syntax, "images_image_1");
    }

    #[test]
    fn compute_dart_usage_syntax_passes_when_requesting_camel_case() {
        // Arrange
        let located_directory = PathBuf::from("assets/images");
        let asset_path = located_directory.join("image_1.png");
        let asset_metadata = AssetMetadata::new(asset_path, located_directory);

        // Act
        let usage_syntax = asset_metadata
            .compute_dart_usage_syntax(&FlutterGenAssetsOutputsStyle::CamelCase)
            .unwrap();

        // Assert
        assert_eq!(usage_syntax, "imagesImage1");
    }

    #[test]
    fn compute_dart_usage_syntax_passes_when_requesting_dot_delimited() {
        // Arrange
        let located_directory = PathBuf::from("assets/images");
        let asset_path = located_directory.join("image_1.png");
        let asset_metadata = AssetMetadata::new(asset_path, located_directory);

        // Act
        let usage_syntax = asset_metadata
            .compute_dart_usage_syntax(&FlutterGenAssetsOutputsStyle::DotDelimited)
            .unwrap();

        // Assert

        assert_eq!(usage_syntax, "images.image1");
    }

    #[test]
    fn compute_dart_usage_syntax_passes_when_asset_is_placed_under_assets_root_directory() {
        // Arrange
        let located_directory = PathBuf::from("assets");
        let asset_path = located_directory.join("image_1.png");
        let asset_metadata = AssetMetadata::new(asset_path, located_directory);

        // Act
        let usage_syntax = asset_metadata
            .compute_dart_usage_syntax(&FlutterGenAssetsOutputsStyle::SnakeCase)
            .unwrap();

        // Assert
        assert_eq!(usage_syntax, "image_1");
    }
}
