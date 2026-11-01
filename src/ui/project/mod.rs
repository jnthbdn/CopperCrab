use serde::{Deserialize, Serialize};

mod export;
mod operation;
mod source;

pub(crate) use operation::{Operation, OperationKind, SelectedTool};
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

    next_id_operation: usize,
}

impl SourceFile {
    pub(crate) fn new(name: String, source: SourceKind) -> Self {
        Self {
            name,
            is_visible: true,
            source,
            operations: Default::default(),
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
}
