use egui::DragValue;
use elegance::{Accent, Button, TextInput};
use rust_i18n::t;

use crate::{
    core::geometry::Point2d,
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
                        Button::new(format!(
                            "{} {}",
                            egui_phosphor::regular::FLIP_HORIZONTAL,
                            t!("ui.button.mirror_x")
                        )),
                    )
                    .clicked()
                {
                    file.source.mirror_x(None);
                    has_changed = true;
                }

                if ui
                    .add_sized(
                        btn_size,
                        Button::new(format!(
                            "{} {}",
                            egui_phosphor::regular::FLIP_VERTICAL,
                            t!("ui.button.mirror_y")
                        )),
                    )
                    .clicked()
                {
                    file.source.mirror_y(None);
                    has_changed = true;
                }
            });

            ui.add_space(8.0);
            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        btn_size,
                        Button::new(format!("{} X-", egui_phosphor::regular::ARROW_LEFT)),
                    )
                    .clicked()
                {
                    file.source.translate(Point2d::new(-1.0, 0.0));
                    has_changed = true;
                }
                if ui
                    .add_sized(
                        btn_size,
                        Button::new(format!("{} X+", egui_phosphor::regular::ARROW_RIGHT)),
                    )
                    .clicked()
                {
                    file.source.translate(Point2d::new(1.0, 0.0));
                    has_changed = true;
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        btn_size,
                        Button::new(format!("{} Y-", egui_phosphor::regular::ARROW_DOWN)),
                    )
                    .clicked()
                {
                    file.source.translate(Point2d::new(0.0, -1.0));
                    has_changed = true;
                }
                if ui
                    .add_sized(
                        btn_size,
                        Button::new(format!("{} Y+", egui_phosphor::regular::ARROW_UP)),
                    )
                    .clicked()
                {
                    file.source.translate(Point2d::new(0.0, 1.0));
                    has_changed = true;
                }
            });

            ui.add_space(8.0);

            if ui
                .add_sized(
                    [btn_size[0] * 2.0 + 8.0, btn_size[1]],
                    Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::CROSSHAIR,
                        t!("ui.button.center_origin")
                    )),
                )
                .clicked()
            {
                let mut bb = crate::core::bounding_box::BoundingBox::new();
                match &file.source {
                    crate::ui::project::SourceKind::Copper(g)
                    | crate::ui::project::SourceKind::Outline(g) => {
                        bb.update_from_bounds(&g.traces.bounds());
                    }
                    crate::ui::project::SourceKind::Drill(d) => {
                        for h in &d.holes {
                            bb.update_from_point(h.x, h.y);
                        }
                    }
                }
                if !bb.is_infinite() {
                    let c = bb.get_center();
                    file.source.translate(Point2d::new(-c.x, -c.y));
                    has_changed = true;
                }
            }

            ui.add_space(8.0);
            ui.separator();

            title_label(ui, &t!("ui.label.position"), false);

            let mut pos_x = 0.0;
            let mut pos_y = 0.0;

            ui.horizontal(|ui| {
                ui.label("X:");
                ui.add(DragValue::new(&mut pos_x).speed(0.1).suffix(" mm"));
                ui.label("Y:");
                ui.add(DragValue::new(&mut pos_y).speed(0.1).suffix(" mm"));
            });

            if ui
                .add_sized(
                    [btn_size[0] * 2.0 + 8.0, btn_size[1]],
                    Button::new(format!("{} Apply", egui_phosphor::regular::CHECK))
                        .accent(Accent::Green),
                )
                .clicked()
            {
                file.source.translate(Point2d::new(pos_x, pos_y));
                has_changed = true;
            }

            ui.add_space(8.0);
            ui.separator();

            if ui
                .add_sized(
                    [btn_size[0] * 2.0 + 8.0, btn_size[1]],
                    Button::new(&format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS_CIRCLE,
                        add_op_text
                    ))
                    .accent(Accent::Green),
                )
                .clicked()
            {
                action = Some(FileTreeActions::AddOperation(id));
            }

            ui.add_space(8.0);

            if ui
                .add_sized(
                    [btn_size[0] * 2.0 + 8.0, btn_size[1]],
                    Button::new(&format!(
                        "{} {}",
                        egui_phosphor::regular::TRASH,
                        t!("ui.button.delete_file")
                    ))
                    .accent(Accent::Red),
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
