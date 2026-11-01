use rust_i18n::t;

use crate::core::{DrillLayer, geometry::Point2d, toolpath::ToolpathGenerator};

#[derive(Debug)]
pub(crate) enum SourceKind {
    Copper(ToolpathGenerator),
    Outline(ToolpathGenerator),
    Drill(DrillLayer),
}

impl SourceKind {
    pub(crate) fn icon(&self) -> &'static str {
        match self {
            SourceKind::Copper(_) => egui_phosphor::regular::GIT_MERGE,
            SourceKind::Outline(_) => egui_phosphor::regular::SQUARE,
            SourceKind::Drill(_) => egui_phosphor::regular::CIRCLES_THREE_PLUS,
        }
    }

    pub(crate) fn operation_prefix(&self) -> &'static str {
        match self {
            SourceKind::Copper(_) => "Isolation",
            SourceKind::Outline(_) => "Outline",
            SourceKind::Drill(_) => "Drill",
        }
    }

    pub(crate) fn add_operation_label(&self) -> String {
        match self {
            SourceKind::Copper(_) => t!("ui.button.add_isolation_operation").to_string(),
            SourceKind::Outline(_) => t!("ui.button.add_outline_operation").to_string(),
            SourceKind::Drill(_) => t!("ui.button.add_drill_operation").to_string(),
        }
    }

    pub(crate) fn as_copper(&self) -> Option<&ToolpathGenerator> {
        match self {
            SourceKind::Copper(g) => Some(g),
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
            SourceKind::Copper(toolpath_generator) | SourceKind::Outline(toolpath_generator) => {
                toolpath_generator.mirror_x(center_x)
            }
            SourceKind::Drill(drill_layer) => drill_layer.mirror_x(center_x),
        }
    }

    pub(crate) fn mirror_y(&mut self, center_y: Option<f64>) {
        match self {
            SourceKind::Copper(toolpath_generator) | SourceKind::Outline(toolpath_generator) => {
                toolpath_generator.mirror_y(center_y)
            }
            SourceKind::Drill(drill_layer) => drill_layer.mirror_y(center_y),
        }
    }

    pub(crate) fn translate(&mut self, offset: Point2d) {
        match self {
            SourceKind::Copper(toolpath_generator) | SourceKind::Outline(toolpath_generator) => {
                toolpath_generator.translate(offset)
            }
            SourceKind::Drill(drill_layer) => drill_layer.translate(offset),
        }
    }
}
