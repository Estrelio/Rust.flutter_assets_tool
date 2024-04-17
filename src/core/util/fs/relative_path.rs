use std::fs;
use std::path::PathBuf;

pub fn relative_path(from: &PathBuf, to: &PathBuf) -> Option<PathBuf> {
    let from_abs = fs::canonicalize(from).ok()?;
    let to_abs = fs::canonicalize(to).ok()?;

    to_abs.strip_prefix(from_abs).ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use crate::core::testing_util::artifact::get_test_artifact_directory;

    use super::*;

    #[test]
    fn can_get_relative_path() {
        // Arrange
        let test_artifact_directory = get_test_artifact_directory().unwrap();
        let test_relative_directory_name = "test_relative_path";
        let test_relative_path = test_artifact_directory.join(test_relative_directory_name);
        fs::create_dir_all(test_relative_path.clone()).unwrap();
        let from = test_artifact_directory;
        let to = test_relative_path;

        // Act
        let result = relative_path(&from, &to);

        // Assert
        assert_eq!(result, Some(PathBuf::from(test_relative_directory_name)));
    }
}