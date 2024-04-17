use std::path::PathBuf;
use recase::ReCase;

use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputsStyle;

pub struct AssetMetadata {
    asset_path: PathBuf,
    located_directory: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum AssetMetadataError {
    /*#[error("The path: {path} is not an absolute path.")]
    NotAbsolutePathError {
        path: PathBuf
    },*/
    #[error("Failed to retrieve the file name from {path}.")]
    RetrieveFileStemError {
        path: PathBuf,
    },
}

impl AssetMetadata {
    pub fn new(
        asset_path: PathBuf,
        located_directory: PathBuf,
    ) -> Self {
        /*fn ensure_absolute_path(path: PathBuf) -> Result<PathBuf,AssetMetadataError >{
            if path.is_absolute() {
                Ok(path)
            } else {
                Err(AssetMetadataError::NotAbsolutePathError{
                    path
                })
            }
        }
*/
        Self {
            asset_path,
            located_directory,
        }
    }
    pub fn get_located_directory(&self) -> &PathBuf {
        &self.located_directory
    }

    pub fn get_asset_path(&self) -> &PathBuf {
        &self.asset_path
    }

    /// Compute a String of the possible usage of the asset in the Dart project.
    pub fn compute_dart_usage_syntax(&self, flutter_gen_assets_outputs_style: FlutterGenAssetsOutputsStyle) -> Result<String, AssetMetadataError> {
        let file_stem = self.asset_path.file_stem().ok_or_else(|| AssetMetadataError::RetrieveFileStemError { path: self.asset_path.clone() })?.to_string_lossy().to_string();
        let relative_asset_path = self.located_directory.join(file_stem);
        let relative_asset_path: PathBuf = relative_asset_path.components()
            // `flutter_gen` doesn't support placing assets outside the Flutter recommended 
            // `assets` directory. Plus, `flutter_gen` doesn't include the `assets` 
            // directory in its generated code.
            .skip(1)
            .collect();
        let recase = ReCase::new(relative_asset_path.to_string_lossy().to_string());
        match flutter_gen_assets_outputs_style {
            FlutterGenAssetsOutputsStyle::SnakeCase => {
                /*let mut usage_syntax: String = String::new();
                for component in file_components {
                    usage_syntax.push_str(&component.to_string_lossy());
                    usage_syntax.push_str("_");
                }
                usage_syntax.push_str(&file_stem);*/
                Ok(recase.snake_case())
            }
            FlutterGenAssetsOutputsStyle::CamelCase => {
                Ok(recase.camel_case())
            }
            FlutterGenAssetsOutputsStyle::DotDelimited => {
                Ok(recase.dot_case())
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
        let usage_syntax = asset_metadata.compute_dart_usage_syntax(FlutterGenAssetsOutputsStyle::SnakeCase).unwrap();
        
        // Assert
        assert_eq!(usage_syntax, "images_image_1");
    }
}
