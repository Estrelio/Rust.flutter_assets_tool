use once_cell::sync::Lazy;
use regex::{Regex, RegexSet};

use crate::core::asset_metadata::{AssetMetadata, AssetMetadataError};
use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputsStyle;

pub const ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME: &str = "assetPath";
pub static ASSETS_PREFIXED_ASSET_USAGE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"R(\s*)\.(\s*)assets_(?<assetPath>\w+)"#).unwrap());

pub fn get_asset_usage_regex(class_name: &str) -> Regex {
    Regex::new(&format!(
        "{class_name}{regex_text}",
        regex_text = r#"(\s*)\.(\s*)(?<assetPath>\w+)\.path"#,
    ))
    .unwrap()
}

#[derive(Debug, thiserror::Error)]
pub enum ComputeAssetUsageRegexSetError {
    #[error("Error while creating regex set: {0}")]
    RegexSetError(#[from] regex::Error),
    #[error("Error while computing asset usage regex set: {0}")]
    AssetMetadataError(#[from] AssetMetadataError),
}

pub fn compute_asset_usage_regex_set_group(
    class_name: &str,
    asset_metadatum: &[AssetMetadata],
    style: FlutterGenAssetsOutputsStyle,
) -> Result<AssetUsageRegexSetGroup, ComputeAssetUsageRegexSetError> {
    let pattern_results = asset_metadatum
        .iter()
        .map(|asset_metadata| {
            let asset_name = asset_metadata
                .compute_dart_usage_syntax(&style)?
                .to_string();
            Ok((
                asset_metadata.to_owned(),
                format!("{class_name}.{asset_name}.path",)
                    .split('.')
                    .collect::<Vec<&str>>()
                    .join(r#"(\s*)\.(\s*)"#),
            ))
        })
        .collect::<Vec<Result<(AssetMetadata, String), AssetMetadataError>>>();
    let mut patterns: Vec<String> = Vec::new();
    let mut regexes: Vec<(Regex, AssetMetadata)> = Vec::new();
    for pattern_result in pattern_results {
        let (asset_usage_asset_name, pattern_result) = pattern_result?;
        patterns.push(pattern_result.to_owned());
        regexes.push((
            Regex::new(pattern_result.as_str())?,
            asset_usage_asset_name.to_owned(),
        ));
    }
    Ok(AssetUsageRegexSetGroup {
        regex_set: RegexSet::new(patterns.iter())?,
        regexes,
    })
}

#[derive(Debug, Clone)]
pub struct AssetUsageRegexSetGroup {
    pub regex_set: RegexSet,
    pub regexes: Vec<(Regex, AssetMetadata)>,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn asset_usage_regex_can_work() {
        // Arrange
        let image_1 = "image_1";
        let image_2 = "image_2";
        let image_3 = "image_3";
        let code = format!(
            "
            final List<String> images = [
                R.{image_1}.path,
                R
                    .{image_2}.path,
                R.
                    {image_3}.path,
            ];
        "
        )
        .to_string();

        // Act
        let matches: Vec<_> = get_asset_usage_regex("R")
            .captures_iter(code.as_str())
            .map(|capture| {
                capture
                    .name(ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME)
                    .unwrap()
                    .as_str()
            })
            .collect();

        // Assert
        assert_eq!(matches.len(), 3);

        assert_eq!(matches, vec![image_1, image_2, image_3]);
    }

    #[test]
    fn assets_prefixed_asset_usage_regex_can_work() {
        // Arrange
        let image_1 = "image_1";
        let image_2 = "image_2";
        let image_3 = "image_3";
        let code = format!(
            "
            final List<String> images = [
                R.assets_{image_1},
                R
                    .assets_{image_2},
                R.
                    assets_{image_3},
            ];
        "
        )
        .to_string();

        // Act
        let matches: Vec<_> = ASSETS_PREFIXED_ASSET_USAGE_REGEX
            .captures_iter(code.as_str())
            .map(|capture| {
                capture
                    .name(ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME)
                    .unwrap()
                    .as_str()
            })
            .collect();

        // Assert
        assert_eq!(matches.len(), 3);

        assert_eq!(matches, vec![image_1, image_2, image_3]);
    }

    #[test]
    fn compute_asset_usage_regex_set_can_work() {
        // Arrange
        let located_directory = PathBuf::from("assets/images");
        let asset_metadata_1 = AssetMetadata::new(
            located_directory.join("image_1.png"),
            located_directory.clone(),
        );
        let asset_metadata_2 = AssetMetadata::new(
            located_directory.join("image_2.png"),
            located_directory.clone(),
        );
        let asset_metadata_3 = AssetMetadata::new(
            located_directory.join("image_3.png"),
            located_directory.clone(),
        );
        let asset_metadatum = vec![
            asset_metadata_1.to_owned(),
            asset_metadata_2.to_owned(),
            asset_metadata_3.to_owned(),
        ];
        let image_1_usage_syntax = asset_metadata_1
            .compute_dart_usage_syntax(&FlutterGenAssetsOutputsStyle::DotDelimited)
            .unwrap();
        let image_2_usage_syntax = asset_metadata_2
            .compute_dart_usage_syntax(&FlutterGenAssetsOutputsStyle::DotDelimited)
            .unwrap();
        let image_3_usage_syntax = asset_metadata_3
            .compute_dart_usage_syntax(&FlutterGenAssetsOutputsStyle::DotDelimited)
            .unwrap();
        let code = format!(
            "
            final List<String> images = [
                R.{image_1}.path,
                R
                    .{image_2}.path,
                R.
                    {image_3}.path,
            ];
",
            image_1 = image_1_usage_syntax,
            image_2 = image_2_usage_syntax,
            image_3 = image_3_usage_syntax,
        );

        // Act
        let regex_set_group = compute_asset_usage_regex_set_group(
            "R",
            &asset_metadatum,
            FlutterGenAssetsOutputsStyle::DotDelimited,
        )
        .unwrap();
        let regexes = regex_set_group.regexes;
        let matches = regexes
            .into_iter()
            .map(|(regex, asset_usage_asset_name)| {
                if regex.is_match(code.as_str()) {
                    Some(asset_usage_asset_name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(regex_set_group.regex_set.matches(code.as_str()).len(), 3);
        assert_eq! {
            matches,
            vec![
                Some(asset_metadata_1),
                Some(asset_metadata_2),
                Some(asset_metadata_3),
            ]
        };
    }
}
