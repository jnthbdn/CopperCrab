use egui::DragValue;
use rust_i18n::t;

use crate::{
    core::geometry::Point2d,
    ui::{
        CopperCrabApp,
        widget::labels::{dragvalue_with_label, title_label},
    },
};

impl CopperCrabApp {
    pub(super) fn ui_right_project(&mut self, ui: &mut egui::Ui) {
        let mut has_changed = false;

        title_label(ui, &t!("ui.label.project"), false);

        dragvalue_with_label(
            ui,
            DragValue::new(&mut self.app_config.common_parameters.z_safe)
                .range(0.1..=10.0)
                .speed(0.1)
                .suffix(" mm"),
            t!("ui.label.parameters.common.safe_z"),
            t!("ui.tooltip.parameters.common.safe_z"),
        );

        ui.add_space(8.0);

        dragvalue_with_label(
            ui,
            DragValue::new(&mut self.app_config.common_parameters.z_finish)
                .range(0..=500)
                .speed(0.1)
                .suffix(" mm"),
            t!("ui.label.parameters.common.final_z"),
            t!("ui.tooltip.parameters.common.final_z"),
        );

        ui.separator();

        if ui
            .add(
                elegance::Button::new(format!(
                    "{} {}",
                    egui_phosphor::regular::CROSSHAIR_SIMPLE,
                    t!("ui.button.move_origin")
                ))
                .full_width(),
            )
            .clicked()
        {
            let b_box = self.get_bounding_box();
            let offset = Point2d::new(-b_box.min.x, -b_box.min.y);

            for file in self.files.iter_mut() {
                file.source.translate(offset.clone());
            }

            has_changed = true;
        }

        if ui
            .add(
                elegance::Button::new(format!(
                    "{} {}",
                    egui_phosphor::regular::FLIP_HORIZONTAL,
                    t!("ui.button.mirror_x")
                ))
                .full_width(),
            )
            .clicked()
        {
            let center = self.get_bounding_box().get_center();

            for file in self.files.iter_mut() {
                file.source.mirror_x(Some(center.x));
            }

            has_changed = true;
        }

        if ui
            .add(
                elegance::Button::new(format!(
                    "{} {}",
                    egui_phosphor::regular::FLIP_VERTICAL,
                    t!("ui.button.mirror_y")
                ))
                .full_width(),
            )
            .clicked()
        {
            let center = self.get_bounding_box().get_center();

            for file in self.files.iter_mut() {
                file.source.mirror_y(Some(center.y));
            }
            has_changed = true;
        }

        if has_changed {
            for id in 0..self.files.len() {
                self.generate_file_toolpaths(id);
            }

            self.center_view_to_board();
        }
    }
}
