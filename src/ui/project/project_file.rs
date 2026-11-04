use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ui::project::{CommonParameters, Operation};

pub(crate) const PROJECT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) enum SourceKindTag {
    Copper,
    Outline,
    Drill,
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