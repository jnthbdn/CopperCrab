use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::{bounding_box::BoundingBox, geometry::Point2d};

mod export;
mod operation;
mod project_file;
mod source;

pub(crate) use operation::{Operation, OperationKind, SelectedTool};
pub(crate) use project_file::{
    PROJECT_VERSION, ProjectFile, SourceFileSnapshot, SourceKindTag, SourceTransform,
};
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

    pub(crate) selected_primitives: HashSet<(usize, usize)>,

    pub(crate) hidden_primitives: HashSet<(usize, usize)>,


    pub(crate) transform: SourceTransform,
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
            selected_primitives: HashSet::new(),
            hidden_primitives: HashSet::new(),
            transform: SourceTransform::default(),
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


    pub(crate) fn translate(&mut self, offset: Point2d) {
        self.source.translate(offset.clone());
        self.transform.translate_x += offset.x;
        self.transform.translate_y += offset.y;
    }


    pub(crate) fn mirror_x(&mut self, center_x: Option<f64>) {
        let cx_current = match center_x {
            Some(c) => c,
            None => {
                let mut bb = BoundingBox::new();
                self.source.update_bounding_box(&mut bb);
                if bb.is_infinite() {
                    return;
                }
                bb.get_center().x
            }
        };

    
        self.source.mirror_x(Some(cx_current));

    
    
    
    
    
    
    
    
    
    
    
        //
    
    
    
        let cx_original = cx_current - self.transform.translate_x;

        self.transform.mirror_x_center = match self.transform.mirror_x_center {
            None => Some(cx_original),
            Some(prev) => {
            
            
            
            
            
            
            
                Some(prev + (cx_original - prev))
            }
        };

    
    
    
    
    }


    pub(crate) fn mirror_y(&mut self, center_y: Option<f64>) {
        let cy_current = match center_y {
            Some(c) => c,
            None => {
                let mut bb = BoundingBox::new();
                self.source.update_bounding_box(&mut bb);
                if bb.is_infinite() {
                    return;
                }
                bb.get_center().y
            }
        };

        self.source.mirror_y(Some(cy_current));

        let cy_original = cy_current - self.transform.translate_y;

        self.transform.mirror_y_center = match self.transform.mirror_y_center {
            None => Some(cy_original),
            Some(prev) => Some(prev + (cy_original - prev)),
        };
    }

    pub(crate) fn to_snapshot(&self) -> Option<SourceFileSnapshot> {
        let path = self.path.clone()?;
        let kind = match &self.source {
            SourceKind::Copper { .. } => SourceKindTag::Copper,
            SourceKind::Outline(_) => SourceKindTag::Outline,
            SourceKind::Drill(_) => SourceKindTag::Drill,
        };

        let hidden_primitives_vec: Vec<(usize, usize)> =
            self.hidden_primitives.iter().copied().collect();

        Some(SourceFileSnapshot {
            name: self.name.clone(),
            is_visible: self.is_visible,
            path,
            kind,
            next_id_operation: self.next_id_operation,
            operations: self.operations.clone(),
            hidden_primitives: hidden_primitives_vec,
            transform: self.transform.clone(),
        })
    }
}
