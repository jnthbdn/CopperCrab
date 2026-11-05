use std::path::{Path, PathBuf};

use rust_i18n::t;

use serde::{Deserialize, Serialize};

use crate::ui::CommonParameters;

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AppTheme {
    #[default]
    Slate,
    Charcoal,
    Frost,
    Paper,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip)]
    file_path: PathBuf,

    #[serde(default = "default_height")]
    pub window_height: f32,

    #[serde(default = "default_width")]
    pub window_width: f32,

    #[serde(default = "default_locale")]
    pub app_locale: String,

    #[serde(default = "default_theme")]
    pub app_theme: AppTheme,

    #[serde(default = "default_true")]
    pub show_grid: bool,

    #[serde(default = "default_true")]
    pub show_axes: bool,


    #[serde(default = "default_grid_step")]
    pub grid_step_mm: f64,

    pub common_parameters: CommonParameters,
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
                Ok(()) => log::info!(
                    "{}",
                    t!(
                        "app_config.info.saved",
                        path = self.file_path.to_string_lossy()
                    )
                ),
                Err(e) => log::error!("{}", t!("app_config.error.fail_save", e = e.to_string())),
            },
            Err(e) => log::error!("{}", t!("app_config.error.fail_save", e = e.to_string())),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            file_path: Default::default(),
            window_height: default_height(),
            window_width: default_width(),
            app_locale: sys_locale::get_locale().unwrap_or("en".to_string()),
            common_parameters: Default::default(),
            app_theme: Default::default(),
            show_grid: true,
            show_axes: true,
            grid_step_mm: default_grid_step(),
        }
    }
}

fn default_width() -> f32 {
    1200.0
}

fn default_height() -> f32 {
    700.0
}

fn default_locale() -> String {
    "en".to_string()
}

fn default_theme() -> AppTheme {
    AppTheme::default()
}

fn default_true() -> bool {
    true
}

fn default_grid_step() -> f64 {
    5.0
}

/// Доступные шаги сетки (мм)
pub const GRID_STEPS: &[f64] = &[1.0, 2.5, 5.0, 10.0, 25.0];
