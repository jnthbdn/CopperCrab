use egui::DragValue;
use elegance::{Accent, Button, TextInput};
use rust_i18n::t;

use crate::{
    core::{bounding_box::BoundingBox, geometry::Point2d},
    ui::{
        CopperCrabApp,
        panels::left::{FileOperationId, FileTreeActions},
        widget::labels::title_label,
    },
};

impl CopperCrabApp {
    pub(super) fn ui_right_file(
        &mut self,
        ui: &mut egui::Ui,
        id: usize,
    ) -> Option<FileTreeActions> {
        let mut action: Option<FileTreeActions> = None;
        let mut has_changed = false;

        let btn_size = [100.0, ui.spacing().interact_size.y];

        title_label(ui, &t!("ui.label.file"), false);
        if let Some(file) = self.files.get_mut(id) {
            let add_op_text = file.source.add_operation_label();

            ui.add(TextInput::new(&mut file.name).label(t!("ui.label.filename")));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::new(format!(
                            "{} {}",
                            egui_phosphor::regular::FLIP_HORIZONTAL,
                            t!("ui.button.mirror_x")
                        )),
                    )
                    .clicked()
                {
                    file.mirror_x(None);
                    has_changed = true;
                }

                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::new(format!(
                            "{} {}",
                            egui_phosphor::regular::FLIP_VERTICAL,
                            t!("ui.button.mirror_y")
                        )),
                    )
                    .clicked()
                {
                    file.mirror_y(None);
                    has_changed = true;
                }
            });

            ui.add_space(8.0);
            ui.separator();

        
            ui.horizontal(|ui| {
                ui.label(t!("ui.label.parameters.common.move_step"));
                ui.add(
                    DragValue::new(&mut self.app_config.common_parameters.move_step)
                        .speed(0.1)
                        .range(0.001..=1000.0)
                        .suffix(" mm"),
                )
                .on_hover_text(t!("ui.tooltip.parameters.common.move_step"));
            });
            ui.add_space(8.0);

            let step = self.app_config.common_parameters.move_step;

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::new(format!("{} X-", egui_phosphor::regular::ARROW_LEFT)),
                    )
                    .clicked()
                {
                    file.translate(Point2d::new(-step, 0.0));
                    has_changed = true;
                }
                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::new(format!("{} X+", egui_phosphor::regular::ARROW_RIGHT)),
                    )
                    .clicked()
                {
                    file.translate(Point2d::new(step, 0.0));
                    has_changed = true;
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::new(format!("{} Y-", egui_phosphor::regular::ARROW_DOWN)),
                    )
                    .clicked()
                {
                    file.translate(Point2d::new(0.0, -step));
                    has_changed = true;
                }
                if ui
                    .add_sized(
                        btn_size,
                        egui::Button::new(format!("{} Y+", egui_phosphor::regular::ARROW_UP)),
                    )
                    .clicked()
                {
                    file.translate(Point2d::new(0.0, step));
                    has_changed = true;
                }
            });

            ui.add_space(8.0);

            if ui
                .add_sized(
                    [btn_size[0] * 2.0 + 8.0, btn_size[1]],
                    egui::Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::CROSSHAIR,
                        t!("ui.button.center_origin")
                    )),
                )
                .clicked()
            {
                let mut bb = BoundingBox::new();
                file.source.update_bounding_box(&mut bb);
                if !bb.is_infinite() {
                    let c = bb.get_center();
                    file.translate(Point2d::new(-c.x, -c.y));
                    has_changed = true;
                }
            }

            ui.add_space(8.0);
            ui.separator();

            title_label(ui, &t!("ui.label.position"), false);

        
            let mut bb = BoundingBox::new();
            file.source.update_bounding_box(&mut bb);

            let (mut pos_x, mut pos_y) = if bb.is_infinite() {
                (0.0, 0.0)
            } else {
                let c = bb.get_center();
                (c.x, c.y)
            };

            let mut pos_changed = false;

            ui.horizontal(|ui| {
                ui.label("X:");
                let resp = ui.add(DragValue::new(&mut pos_x).speed(0.1).suffix(" mm"));
                pos_changed |= resp.changed();

                ui.label("Y:");
                let resp = ui.add(DragValue::new(&mut pos_y).speed(0.1).suffix(" mm"));
                pos_changed |= resp.changed();
            });

        
            if pos_changed && !bb.is_infinite() {
                let c = bb.get_center();
                file.translate(Point2d::new(pos_x - c.x, pos_y - c.y));
                has_changed = true;
            }

            ui.add_space(8.0);
            ui.separator();

            if ui
                .add_sized(
                    [btn_size[0] * 2.0 + 8.0, btn_size[1]],
                    egui::Button::new(&format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS_CIRCLE,
                        add_op_text
                    )),
                )
                .clicked()
            {
                action = Some(FileTreeActions::AddOperation(id));
            }

            ui.add_space(8.0);

            if ui
                .add_sized(
                    [btn_size[0] * 2.0 + 8.0, btn_size[1]],
                    egui::Button::new(&format!(
                        "{} {}",
                        egui_phosphor::regular::TRASH,
                        t!("ui.button.delete_file")
                    )),
                )
                .clicked()
            {
                action = Some(FileTreeActions::DeleteItem(FileOperationId::File(id)));
            }

            if has_changed {
                self.generate_file_toolpaths(id);
            }
        }

        action
    }
}
