use once_cell::sync::Lazy;
use regex::Regex;

pub const ASSET_USAGE_REGEX_ASSET_PATH_GROUP_NAME: &str = "assetPath";
pub static ASSET_USAGE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"R(\s*)\.(\s*)(?<assetPath>\w+)"#).unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_usage_regex_can_work() {
        // Arrange
        let image_1 = "image_1";
        let image_2 = "image_2";
        let image_3 = "image_3";
        let code = format!("
            final List<String> images = [
                R.{image_1},
                R
                    .{image_2},
                R.
                    {image_3},
            ];
        ")
            .to_string();

        // Act
        let matches: Vec<_> = ASSET_USAGE_REGEX
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

        assert_eq!(
            matches,
            vec![image_1, image_2, image_3]
        );
    }
}
