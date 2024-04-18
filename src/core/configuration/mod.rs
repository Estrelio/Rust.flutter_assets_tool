pub mod flutter_assets_tool {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};
    use crate::commands;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub struct FlutterAssetsTool {
        /// The paths to ignore when searching for unused assets.
        ///
        /// See [`subcommand`].
        ///
        /// [`subcommand`]: commands::cli::SubCommands::ListUnused
        list_unused_ignore_paths: Option<Vec<String>>,
    }

    impl FlutterAssetsTool {
        pub fn get_ignore_paths(&self) -> Option<Vec<String>> {
            self.list_unused_ignore_paths.to_owned()
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ReadFlutterAssetsToolFileError {
        #[error("Failed to open flutter_assets_tool.yaml file.")]
        OpenFileError {
            #[from]
            source: std::io::Error,
        },
        #[error("Failed to parse flutter_assets_tool.yaml file.")]
        ParseFileError {
            #[from]
            source: serde_yml::Error,
        },
    }

    const FLUTTER_ASSETS_TOOL_FILE_NAME: &str = "flutter_assets_tool.yaml";

    pub fn read_flutter_assets_tool_file(
        directory: &PathBuf,
    ) -> Result<FlutterAssetsTool, ReadFlutterAssetsToolFileError> {
        let flutter_assets_tool_file_path = directory.join(FLUTTER_ASSETS_TOOL_FILE_NAME);
        let flutter_assets_tool_file = std::fs::File::open(&flutter_assets_tool_file_path)?;
        let flutter_assets_tool: FlutterAssetsTool =
            serde_yml::from_reader(flutter_assets_tool_file)?;

        Ok(flutter_assets_tool)
    }
}
