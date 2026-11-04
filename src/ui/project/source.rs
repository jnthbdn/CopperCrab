use rust_i18n::t;

use crate::core::{
    DrillLayer, PcbLayer, bounding_box::BoundingBox, geometry::Point2d, toolpath::ToolpathGenerator,
};

use super::project_file::SourceTransform;

#[derive(Debug)]
pub(crate) enum SourceKind {
    Copper {
        generator: ToolpathGenerator,
        layer: PcbLayer,
    },
    Outline(ToolpathGenerator),
    Drill(DrillLayer),
}

impl SourceKind {
    pub(crate) fn icon(&self) -> &'static str {
        match self {
            SourceKind::Copper { .. } => egui_phosphor::regular::GIT_MERGE,
            SourceKind::Outline(_) => egui_phosphor::regular::SQUARE,
            SourceKind::Drill(_) => egui_phosphor::regular::CIRCLES_THREE_PLUS,
        }
    }

    pub(crate) fn operation_prefix(&self) -> &'static str {
        match self {
            SourceKind::Copper { .. } => "Isolation",
            SourceKind::Outline(_) => "Outline",
            SourceKind::Drill(_) => "Drill",
        }
    }

    pub(crate) fn add_operation_label(&self) -> String {
        match self {
            SourceKind::Copper { .. } => t!("ui.button.add_isolation_operation").to_string(),
            SourceKind::Outline(_) => t!("ui.button.add_outline_operation").to_string(),
            SourceKind::Drill(_) => t!("ui.button.add_drill_operation").to_string(),
        }
    }

    pub(crate) fn as_copper(&self) -> Option<&ToolpathGenerator> {
        match self {
            SourceKind::Copper { generator, .. } => Some(generator),
            _ => None,
        }
    }

    pub(crate) fn as_copper_mut(&mut self) -> Option<(&mut ToolpathGenerator, &mut PcbLayer)> {
        match self {
            SourceKind::Copper { generator, layer } => Some((generator, layer)),
            _ => None,
        }
    }

    pub(crate) fn copper_layer(&self) -> Option<&PcbLayer> {
        match self {
            SourceKind::Copper { layer, .. } => Some(layer),
            _ => None,
        }
    }

    pub(crate) fn as_outline(&self) -> Option<&ToolpathGenerator> {
        match self {
            SourceKind::Outline(g) => Some(g),
            _ => None,
        }
    }

    pub(crate) fn as_drill(&self) -> Option<&DrillLayer> {
        match self {
            SourceKind::Drill(d) => Some(d),
            _ => None,
        }
    }

    pub(crate) fn mirror_x(&mut self, center_x: Option<f64>) {
        match self {
            SourceKind::Copper { generator, layer } => {
                generator.mirror_x(center_x);
                layer.mirror_x(center_x);
            }
            SourceKind::Outline(toolpath_generator) => toolpath_generator.mirror_x(center_x),
            SourceKind::Drill(drill_layer) => drill_layer.mirror_x(center_x),
        }
    }

    pub(crate) fn mirror_y(&mut self, center_y: Option<f64>) {
        match self {
            SourceKind::Copper { generator, layer } => {
                generator.mirror_y(center_y);
                layer.mirror_y(center_y);
            }
            SourceKind::Outline(toolpath_generator) => toolpath_generator.mirror_y(center_y),
            SourceKind::Drill(drill_layer) => drill_layer.mirror_y(center_y),
        }
    }

    pub(crate) fn translate(&mut self, offset: Point2d) {
        match self {
            SourceKind::Copper { generator, layer } => {
                generator.translate(offset.clone());
                layer.translate(offset);
            }
            SourceKind::Outline(toolpath_generator) => toolpath_generator.translate(offset),
            SourceKind::Drill(drill_layer) => drill_layer.translate(offset),
        }
    }


    pub(crate) fn update_bounding_box(&self, bb: &mut BoundingBox) {
        match self {
            SourceKind::Copper { generator, .. } => {
                bb.update_from_bounds(&generator.traces.bounds());
            }
            SourceKind::Outline(generator) => {
                bb.update_from_bounds(&generator.traces.bounds());
            }
            SourceKind::Drill(layer) => {
                for h in &layer.holes {
                    bb.update_from_point(h.x, h.y);
                }
            }
        }
    }




    pub(crate) fn apply_saved_transform(&mut self, t: &SourceTransform) {
        if let Some(cx) = t.mirror_x_center {
            self.mirror_x(Some(cx));
        }
        if let Some(cy) = t.mirror_y_center {
            self.mirror_y(Some(cy));
        }
        if t.translate_x != 0.0 || t.translate_y != 0.0 {
            self.translate(Point2d::new(t.translate_x, t.translate_y));
        }
    }
}
