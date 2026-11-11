use std::path::PathBuf;

use directories::ProjectDirs;

pub fn get_config_dir() -> Option<PathBuf> {
    // папка, где лежит exe
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("config"))
}
