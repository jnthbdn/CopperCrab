use std::collections::HashSet;
use std::path::PathBuf;

use crate::core::{bounding_box::BoundingBox, geometry::Point2d};

use super::{
    Operation, SourceKind,
    project_file::{SourceFileSnapshot, SourceKindTag, SourceTransform, TransformOp},
};

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
        self.transform.push(TransformOp::Translate {
            dx: offset.x,
            dy: offset.y,
        });
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
        self.transform.push(TransformOp::MirrorX { cx: cx_current });
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
        self.transform.push(TransformOp::MirrorY { cy: cy_current });
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
