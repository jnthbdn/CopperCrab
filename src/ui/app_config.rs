use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ui::{ContextLayer, ContextParameters};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip)]
    file_path: PathBuf,

    #[serde(default = "default_width")]
    pub window_height: f32,

    #[serde(default = "default_height")]
    pub window_width: f32,

    pub context_parameter: ContextParameters,
    pub context_layer: ContextLayer,
}

impl AppConfig {
    pub fn new(config_folder: &Path) -> Self {
        let file_path = config_folder.join("app_config.toml");

        let mut s: AppConfig = std::fs::read_to_string(&file_path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();

        s.file_path = file_path;
        s
    }

    pub fn save(&mut self) {
        match toml::to_string_pretty(self) {
            Ok(content) => match std::fs::write(&self.file_path, content) {
                Ok(()) => log::info!("App config saved to: {}", self.file_path.to_string_lossy()),
                Err(e) => log::error!("Failed to write app config file: {e}"),
            },
            Err(e) => log::error!("Failed to write app config file: {e}"),
        }
    }
}

fn default_width() -> f32 {
    1200.0
}

fn default_height() -> f32 {
    700.0
}
