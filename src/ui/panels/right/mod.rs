use crate::ui::{
    CopperCrabApp,
    panels::left::{FileOperationId, FileTreeActions},
};

mod file;
mod operation;
mod project;

impl CopperCrabApp {
    pub(crate) fn ui_right(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut action: Option<FileTreeActions> = None;

            if let Some(selected) = self.tree_files_states.selected().first() {
                match *selected {
                    FileOperationId::Root => self.ui_right_project(ui),
                    FileOperationId::File(id) => action = self.ui_right_file(ui, id),
                    FileOperationId::Operation(id_file, id_op) => {
                        action = self.ui_right_operation(ui, id_file, id_op)
                    }
                }
            };

            if let Some(action) = action {
                self.handle_action(action);
            }
        });
    }
}
