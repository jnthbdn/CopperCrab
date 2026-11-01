use elegance::{Accent, Button, TextInput};
use rust_i18n::t;

use crate::ui::{
    CopperCrabApp,
    panels::left::{FileOperationId, FileTreeActions},
    widget::labels::title_label,
};

impl CopperCrabApp {
    pub(super) fn ui_right_file(
        &mut self,
        ui: &mut egui::Ui,
        id: usize,
    ) -> Option<FileTreeActions> {
        let mut action: Option<FileTreeActions> = None;
        let mut has_changed = false;

        title_label(ui, &t!("ui.label.file"), false);
        if let Some(file) = self.files.get_mut(id) {
            let add_op_text = file.source.add_operation_label();

            ui.add(TextInput::new(&mut file.name).label(t!("ui.label.filename")));
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
                file.source.mirror_x(None);
                has_changed = true;
            }

            ui.add_space(8.0);

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
                file.source.mirror_y(None);
                has_changed = true;
            }

            ui.add_space(8.0);
            ui.separator();

            if ui
                .add(
                    Button::new(&format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS_CIRCLE,
                        add_op_text
                    ))
                    .full_width()
                    .accent(Accent::Green),
                )
                .clicked()
            {
                action = Some(FileTreeActions::AddOperation(id));
            }

            ui.add_space(8.0);

            if ui
                .add(
                    Button::new(&format!(
                        "{} {}",
                        egui_phosphor::regular::TRASH,
                        t!("ui.button.delete_file")
                    ))
                    .full_width()
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
