use std::path::PathBuf;

use directories::ProjectDirs;

pub fn get_config_dir() -> Option<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("", "jnthbdn", "CopperCrab") {
        Some(proj_dirs.config_dir().into())
    } else {
        None
    }
}
