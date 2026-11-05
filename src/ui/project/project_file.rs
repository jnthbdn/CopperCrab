use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ui::project::{CommonParameters, Operation};

pub(crate) const PROJECT_VERSION: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) enum SourceKindTag {
    Copper,
    Outline,
    Drill,
}

/// Одна операция трансформации исходного файла.
/// Хранится в координатах ТЕКУЩЕГО (уже трансформированного) слоя
/// на момент применения — это позволяет воспроизвести операции 1-в-1.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) enum TransformOp {
    Translate { dx: f64, dy: f64 },
    MirrorX { cx: f64 },
    MirrorY { cy: f64 },
}

/// Накопленная трансформация исходного файла — упорядоченный список операций.
/// Пустой список = тождество.
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
