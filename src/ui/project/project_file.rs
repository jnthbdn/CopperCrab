use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ui::project::{CommonParameters, ExportSettings, Operation};

pub(crate) const PROJECT_VERSION: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) enum SourceKindTag {
    Copper,
    Outline,
    Drill,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum TransformOp {
    Translate { dx: f64, dy: f64 },
    MirrorX { cx: f64 },
    MirrorY { cy: f64 },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub(crate) struct SourceTransform {
    #[serde(default)]
    pub ops: Vec<TransformOp>,
}

impl SourceTransform {
    pub(crate) fn is_identity(&self) -> bool {
        self.ops.is_empty()
    }

    pub(crate) fn push(&mut self, op: TransformOp) {
        if let Some(last) = self.ops.last() {
            match (last, &op) {
                (TransformOp::MirrorX { cx: a }, TransformOp::MirrorX { cx: b })
                    if (a - b).abs() < f64::EPSILON =>
                {
                    self.ops.pop();
                    return;
                }
                (TransformOp::MirrorY { cy: a }, TransformOp::MirrorY { cy: b })
                    if (a - b).abs() < f64::EPSILON =>
                {
                    self.ops.pop();
                    return;
                }
                _ => {}
            }
        }
        self.ops.push(op);
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
    #[serde(default)]
    pub common_parameters: CommonParameters,
    #[serde(default)]
    pub export_settings: ExportSettings,
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
        let mut project: ProjectFile = toml::from_str(&content)?;
        project.export_settings.migrate_legacy();
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
