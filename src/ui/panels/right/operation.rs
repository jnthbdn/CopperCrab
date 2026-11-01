use egui::DragValue;
use elegance::{Accent, Button, Select, TextInput, Theme};
use rust_i18n::t;

use crate::{
    core::tools::{CncTool, library::ToolLibrary},
    ui::{
        CopperCrabApp,
        panels::left::{FileOperationId, FileTreeActions},
        project::{OperationKind, SelectedTool},
        widget::labels::{dragvalue_with_label, title_label},
    },
};

impl CopperCrabApp {
    pub(super) fn ui_right_operation(
        &mut self,
        ui: &mut egui::Ui,
        id_file: usize,
        id_op: usize,
    ) -> Option<FileTreeActions> {
        let mut action: Option<FileTreeActions> = None;
        let theme = Theme::current(ui.ctx());
        let mut has_changed = false;

        if let Some(operation) = self
            .files
            .get_mut(id_file)
            .and_then(|f| f.operations.get_mut(id_op))
        {
            let lib = &self.tools.tool_library;

            title_label(ui, &operation.kind.title(), false);

            ui.add(TextInput::new(&mut operation.name).label(t!("ui.label.operation.name")));
            ui.add_space(8.0);

            let options = Self::tool_options(&operation.kind, lib);
            if ui
                .add(
                    Select::new("operation_tool", &mut operation.tool)
                        .options(options)
                        .width(ui.available_width())
                        .label(operation.kind.tool_select_label()),
                )
                .changed()
            {
                has_changed = true;
            }
            ui.add_space(8.0);

            let (depth_label, depth_tooltip) = operation.kind.depth_texts();
            if dragvalue_with_label(
                ui,
                DragValue::new(&mut operation.depth)
                    .range(operation.kind.depth_range())
                    .speed(0.01)
                    .suffix(" mm"),
                depth_label,
                depth_tooltip,
            )
            .changed()
            {
                has_changed = true;
            }

            match &mut operation.kind {
                OperationKind::Isolation {
                    passes,
                    overlap,
                    offset,
                } => {
                    let iso_width = Self::tool_cutting_width(lib, &operation.tool, operation.depth);

                    ui.label(theme.body_text(format!(
                        "  > {}: {:.2} mm",
                        t!("ui.label.parameters.isolation.depth_result"),
                        iso_width
                    )));
                    ui.add_space(8.0);

                    if dragvalue_with_label(
                        ui,
                        DragValue::new(passes).range(1..=50).speed(1),
                        t!("ui.label.parameters.isolation.passes"),
                        t!("ui.tooltip.parameters.isolation.passes"),
                    )
                    .changed()
                    {
                        has_changed = true;
                    }
                    ui.add_space(8.0);

                    if dragvalue_with_label(
                        ui,
                        DragValue::new(overlap).range(0..=100).speed(1).suffix(" %"),
                        t!("ui.label.parameters.isolation.overlap"),
                        t!("ui.tooltip.parameters.isolation.overlap"),
                    )
                    .changed()
                    {
                        has_changed = true;
                    }
                    ui.add_space(8.0);

                    if dragvalue_with_label(
                        ui,
                        DragValue::new(offset)
                            .range(0.0..=f64::INFINITY)
                            .speed(0.01)
                            .suffix(" mm"),
                        t!("ui.label.parameters.isolation.offset"),
                        t!("ui.tooltip.parameters.isolation.offset"),
                    )
                    .changed()
                    {
                        has_changed = true;
                    }

                    ui.add_space(16.0);
                    ui.label(theme.body_text(format!(
                        "{}: {:.2} mm",
                        t!("ui.label.parameters.isolation.total_width"),
                        iso_width
                            + passes.saturating_sub(1) as f64
                                * iso_width
                                * (1.0 - *overlap / 100.0)
                    )));
                }
                OperationKind::Drill { peck_step } => {
                    ui.add_space(8.0);
                    if dragvalue_with_label(
                        ui,
                        DragValue::new(peck_step)
                            .range(0.1..=10.0)
                            .speed(0.1)
                            .suffix(" mm"),
                        t!("ui.label.parameters.drill.drill_peck"),
                        t!("ui.tooltip.parameters.drill_peck"),
                    )
                    .changed()
                    {
                        has_changed = true;
                    }
                }
                OperationKind::Outline => {}
            }

            ui.add_space(16.0);
            if ui
                .add(
                    Button::new(&format!(
                        "{} {}",
                        egui_phosphor::regular::TRASH,
                        t!("ui.button.delete_operation")
                    ))
                    .full_width()
                    .accent(Accent::Red),
                )
                .clicked()
            {
                action = Some(FileTreeActions::DeleteItem(FileOperationId::Operation(
                    id_file, id_op,
                )));
            }
        }

        if has_changed {
            self.generate_operation_toolpaths(id_file, id_op);
        }

        action
    }

    fn tool_options(kind: &OperationKind, lib: &ToolLibrary) -> Vec<(SelectedTool, String)> {
        let mut out = Vec::new();
        if matches!(kind, OperationKind::Isolation { .. }) {
            out.extend(
                lib.vbits
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (SelectedTool::VBit(i), t.to_string())),
            );
        }

        if matches!(
            kind,
            OperationKind::Isolation { .. } | OperationKind::Outline
        ) {
            out.extend(
                lib.end_mills
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (SelectedTool::EndMill(i), t.to_string())),
            );
        }

        if matches!(kind, OperationKind::Drill { .. }) {
            out.extend(
                lib.drill_bits
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (SelectedTool::Drill(i), t.to_string())),
            );
        }
        out
    }

    fn tool_cutting_width(lib: &ToolLibrary, tool: &SelectedTool, depth: f64) -> f64 {
        match tool {
            SelectedTool::VBit(i) => lib.vbits.get(*i).map_or(0.0, |t| t.cutting_width(depth)),
            SelectedTool::EndMill(i) => lib
                .end_mills
                .get(*i)
                .map_or(0.0, |t| t.cutting_width(depth)),
            _ => 0.0,
        }
    }
}
