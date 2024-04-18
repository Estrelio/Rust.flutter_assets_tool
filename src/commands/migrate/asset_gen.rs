pub mod migrate {
    use std::path::PathBuf;
    use std::sync::Arc;

    use crate::core::asset_usage::{
        ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME, ASSETS_PREFIXED_ASSET_USAGE_REGEX,
    };
    use crate::core::flutter::get_flutter_project_inner_directory_path::get_flutter_project_lib_path;
    use crate::core::flutter::pubspec_yaml::ReadPubspecYamlFileError;
    use crate::core::util::fs::read_file_recursively;

    #[derive(Debug, thiserror::Error)]
    pub enum MigrateAssetGenToFlutterGenError {
        #[error("{0}")]
        ReadPubspecYamlFileError(#[from] ReadPubspecYamlFileError),
        #[error("{0}")]
        ReadFileRecursivelyError(#[from] read_file_recursively::ReadFileRecursivelyError),
    }

    #[derive(Debug, thiserror::Error)]
    pub enum MigrateUsageError {
        #[error("Unable to read file for migration: {0}")]
        ReadFileError(std::io::Error),
        #[error("Unable to write file for migration: {0}")]
        WriteFileError(std::io::Error),
    }

    pub async fn migrate_asset_gen_to_flutter_gen(
        flutter_project_path: &PathBuf,
    ) -> Result<(), MigrateAssetGenToFlutterGenError> {
        let scanning_directory = get_flutter_project_lib_path(&flutter_project_path);

        read_file_recursively::read_file_recursively(
            &scanning_directory,
            Arc::new(|path: PathBuf| async move {
                if path.extension().unwrap_or_default() != "dart" {
                    return Ok(());
                }

                let file_content = tokio::fs::read_to_string(&path).await.map_err(|err| {
                    read_file_recursively::ReadFileRecursivelyError::OtherError {
                        source: Box::new(MigrateUsageError::ReadFileError(err)),
                    }
                })?;

                let new_file_content = migrate_usage(&file_content);

                tokio::fs::write(&path, new_file_content)
                    .await
                    .map_err(
                        |err| read_file_recursively::ReadFileRecursivelyError::OtherError {
                            source: Box::new(MigrateUsageError::WriteFileError(err)),
                        },
                    )?;

                Ok(())
            }),
        )
        .await?;
        Ok(())
    }

    fn migrate_usage(file_content: &String) -> String {
        ASSETS_PREFIXED_ASSET_USAGE_REGEX
            .replace_all(&file_content, |captures: &regex::Captures| -> String {
                let asset_name = captures
                    .name(ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME)
                    .unwrap()
                    .as_str();
                format!("R.{asset_name}.path",)
            })
            .to_string()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn migrate_usage_can_replace_asset_gen_usage_to_flutter_gen_usage() {
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
            let new_code = migrate_usage(&code);

            // Assert
            assert_eq!(
                new_code,
                format!(
                    "
            final List<String> images = [
                R.{image_1}.path,
                R.{image_2}.path,
                R.{image_3}.path,
            ];
        "
                )
                .to_string()
            );
        }
    }
}
