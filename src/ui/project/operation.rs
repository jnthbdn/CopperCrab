use clipper2::Paths;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) enum SelectedTool {
    VBit(usize),
    EndMill(usize),
    Drill(usize),

    #[default]
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum OperationKind {
    Isolation {
        passes: u32,
        overlap: f64,
        offset: f64,
    },
    Outline,
    Drill {
        peck_step: f64,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Operation {
    pub(crate) name: String,
    pub(crate) tool: SelectedTool,
    pub(crate) depth: f64,

  
    #[serde(skip)]
    pub(crate) toolpath: Vec<Paths>,

    pub(crate) kind: OperationKind,
}

impl Operation {
    pub(crate) fn new_isolation(name: String) -> Self {
        Self {
            name,
            tool: SelectedTool::None,
            depth: 0.0,
            toolpath: Vec::new(),
            kind: OperationKind::Isolation {
                passes: 1,
                overlap: 50.0,
                offset: 0.0,
            },
        }
    }

    pub(crate) fn new_outline(name: String) -> Self {
        Self {
            name,
            tool: SelectedTool::None,
            depth: 0.0,
            toolpath: Vec::new(),
            kind: OperationKind::Outline,
        }
    }

    pub(crate) fn new_drill(name: String) -> Self {
        Self {
            name,
            tool: SelectedTool::None,
            depth: 0.0,
            toolpath: Vec::new(),
            kind: OperationKind::Drill { peck_step: 1.0 },
        }
    }
}

impl OperationKind {
    pub(crate) fn title(&self) -> String {
        match self {
            OperationKind::Isolation { .. } => {
                t!("ui.label.parameters.isolation.title").to_string()
            }
            OperationKind::Outline => t!("ui.label.parameters.outline.title").to_string(),
            OperationKind::Drill { .. } => t!("ui.label.parameters.drill.title").to_string(),
        }
    }

    pub(crate) fn tool_select_label(&self) -> String {
        match self {
            OperationKind::Isolation { .. } => t!("ui.select_tool.isolation").to_string(),
            OperationKind::Outline => t!("ui.select_tool.outline").to_string(),
            OperationKind::Drill { .. } => t!("ui.select_tool.drill").to_string(),
        }
    }

    pub(crate) fn depth_texts(&self) -> (String, String) {
        match self {
            OperationKind::Isolation { .. } => (
                t!("ui.label.parameters.isolation.depth").to_string(),
                t!("ui.tooltip.parameters.isolation.depth").to_string(),
            ),
            OperationKind::Outline => (
                t!("ui.label.parameters.outline.depth").to_string(),
                t!("ui.tooltip.parameters.outline.depth").to_string(),
            ),
            OperationKind::Drill { .. } => (
                t!("ui.label.parameters.drill.drill_depth").to_string(),
                t!("ui.tooltip.parameters.drill_depth").to_string(),
            ),
        }
    }

    pub(crate) fn depth_range(&self) -> std::ops::RangeInclusive<f64> {
        match self {
            OperationKind::Isolation { .. } => 0.01..=5.0,
            _ => 0.1..=20.0,
        }
    }
}