use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ui::project::{CommonParameters, Operation};

pub(crate) const PROJECT_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) enum SourceKindTag {
    Copper,
    Outline,
    Drill,
}

/// Накопленная трансформация исходного файла (сдвиг + зеркала).
/// Хранится в координатах ИСХОДНОГО файла (до сдвига).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SourceTransform {
    pub translate_x: f64,
    pub translate_y: f64,


    #[serde(default)]
    pub mirror_x_center: Option<f64>,


    #[serde(default)]
    pub mirror_y_center: Option<f64>,
}

impl Default for SourceTransform {
    fn default() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            mirror_x_center: None,
            mirror_y_center: None,
        }
    }
}

impl SourceTransform {
    pub(crate) fn is_identity(&self) -> bool {
        self.translate_x == 0.0
            && self.translate_y == 0.0
            && self.mirror_x_center.is_none()
            && self.mirror_y_center.is_none()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SourceFileSnapshot {
    pub name: String,
    pub is_visible: bool,
    pub path: PathBuf,
    pub kind: SourceKindTag,
    pub next_id_operation: usize,
    pub operations: Vec<Operation>,

    #[serde(default)]
    pub hidden_primitives: Vec<(usize, usize)>,

    #[serde(default)]
    pub transform: SourceTransform,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ProjectFile {
    pub version: u32,
    pub common_parameters: CommonParameters,
    pub files: Vec<SourceFileSnapshot>,
}

impl ProjectFile {
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let project: ProjectFile = toml::from_str(&content)?;
        if project.version != PROJECT_VERSION {
            log::warn!(
                "Project version mismatch: file={}, app={}",
                project.version,
                PROJECT_VERSION
            );
        }
        Ok(project)
    }
}
