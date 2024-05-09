pub mod migrate {
    use std::path::{Path, PathBuf};

    use crate::core::flutter::pubspec_yaml::FlutterGenAssetsOutputsStyle;
    use crate::core::migrate_flutter_gen_style::MigrateFlutterGenStyleError;

    pub async fn migrate_flutter_gen_style(
        flutter_project_path: &Path,
        ignore_path_bufs: Vec<PathBuf>,
        previous_style: FlutterGenAssetsOutputsStyle,
    ) -> Result<(), MigrateFlutterGenStyleError> {
        crate::core::migrate_flutter_gen_style::migrate_flutter_gen_style(
            flutter_project_path,
            ignore_path_bufs,
            previous_style,
        )
        .await
    }
}
