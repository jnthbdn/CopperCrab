use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod export;
mod operation;
mod project_file;
mod source;

pub(crate) use operation::{Operation, OperationKind, SelectedTool};
pub(crate) use project_file::{ProjectFile, SourceFileSnapshot, SourceKindTag, PROJECT_VERSION};
pub(crate) use source::SourceKind;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CommonParameters {
    pub(crate) z_safe: f64,
    pub(crate) z_finish: f64,
}

impl Default for CommonParameters {
    fn default() -> Self {
        Self {
            z_safe: 3.0,
            z_finish: 5.0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct SourceFile {
    pub name: String,
    pub is_visible: bool,
    pub source: SourceKind,
    pub operations: Vec<Operation>,


    pub path: Option<PathBuf>,


    pub(crate) next_id_operation: usize,
}

impl SourceFile {
    pub(crate) fn new(name: String, source: SourceKind, path: Option<PathBuf>) -> Self {
        Self {
            name,
            is_visible: true,
            source,
            operations: Default::default(),
            path,
            next_id_operation: 0,
        }
    }

    pub(crate) fn next_operation_name(&mut self) -> String {
        self.next_id_operation += 1;
        format!(
            "{} #{}",
            self.source.operation_prefix(),
            self.next_id_operation
        )
    }


    pub(crate) fn to_snapshot(&self) -> Option<SourceFileSnapshot> {
        let path = self.path.clone()?;
        let kind = match &self.source {
            SourceKind::Copper(_) => SourceKindTag::Copper,
            SourceKind::Outline(_) => SourceKindTag::Outline,
            SourceKind::Drill(_) => SourceKindTag::Drill,
        };

        Some(SourceFileSnapshot {
            name: self.name.clone(),
            is_visible: self.is_visible,
            path,
            kind,
            next_id_operation: self.next_id_operation,
            operations: self.operations.clone(),
        })
    }
}