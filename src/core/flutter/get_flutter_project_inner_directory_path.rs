use std::path::PathBuf;

const FLUTTER_PROJECT_LIB_DIRECTORY_NAME: &str = "lib";

pub fn get_flutter_project_lib_path(flutter_project_path: &PathBuf) -> PathBuf {
    flutter_project_path.join(FLUTTER_PROJECT_LIB_DIRECTORY_NAME)
}
