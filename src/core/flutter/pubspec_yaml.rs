use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PubspecYaml {
    /// The Dart project name.
    pub name: String,

    pub flutter: Flutter,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Flutter {
    pub assets: Vec<String>,
}

/// Check https://github.com/FlutterGen/flutter_gen/blob/main/packages/core/lib/settings/config_default.dart for the schema.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FlutterGen {
    pub assets: FlutterGenAssets,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FlutterGenAssets {
    pub outputs: FlutterGenAssetsOutputs,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FlutterGenAssetsOutputs {
    pub style: FlutterGenAssetsOutputsStyle,
    pub class_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum FlutterGenAssetsOutputsStyle {
    SnakeCase,
    CamelCase,
    DotDelimited,
}

#[derive(Debug, thiserror::Error)]
pub enum ReadPubspecYamlFileError {
    #[error("Failed to open pubspec.yaml file in {path}.\n{source}")]
    OpenFileError {
        source: std::io::Error,
        path: PathBuf,
    },
    #[error("Failed to parse pubspec.yaml file.")]
    ParseFileError { source: serde_yml::Error },
}

const PUBSPEC_YAML_FILE_NAME: &str = "pubspec.yaml";

/// Read the pubspec.yaml file from [directory].
///
/// # Arguments
///
/// * `directory`: The directory where the pubspec.yaml file is located.
///
/// returns: Result<PubspecYaml, <ReadPubspecYamlFileError>>
pub fn read_pubspec_yaml_file(
    directory: &PathBuf,
) -> Result<PubspecYaml, ReadPubspecYamlFileError> {
    let pubspec_yaml_file_path = directory.join(PUBSPEC_YAML_FILE_NAME);
    let pubspec_yaml_file = std::fs::File::open(&pubspec_yaml_file_path).map_err(|err| {
        ReadPubspecYamlFileError::OpenFileError {
            source: err,
            path: pubspec_yaml_file_path,
        }
    })?;
    let pubspec_yaml_file_reader = std::io::BufReader::new(pubspec_yaml_file);
    let pubspec_yaml: PubspecYaml = serde_yml::from_reader(pubspec_yaml_file_reader)
        .map_err(|err| ReadPubspecYamlFileError::ParseFileError { source: err })?;

    Ok(pubspec_yaml)
}
