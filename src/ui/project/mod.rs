use serde::{Deserialize, Serialize};

mod export;
mod export_settings;
mod operation;
mod project_file;
mod source;
mod source_file;

pub(crate) use export_settings::{ExportFormat, ExportSettings};
pub(crate) use operation::{Operation, OperationKind, SelectedTool};
pub(crate) use project_file::{
    PROJECT_VERSION, ProjectFile, SourceFileSnapshot, SourceKindTag, SourceTransform, TransformOp,
};
pub(crate) use source::SourceKind;
pub(crate) use source_file::SourceFile;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CommonParameters {
    pub(crate) z_safe: f64,
    pub(crate) z_finish: f64,
    #[serde(default = "default_move_step")]
    pub(crate) move_step: f64,
}

impl Default for CommonParameters {
    fn default() -> Self {
        Self {
            z_safe: 3.0,
            z_finish: 5.0,
            move_step: 1.0,
        }
    }
}

fn default_move_step() -> f64 {
    1.0
}
