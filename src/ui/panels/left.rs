use egui::{Align, Id, Layout, Ui};
use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder};
use elegance::{Accent, Button, Switch};
use log::info;
use rust_i18n::t;

use crate::{
    core::{
        parser::{excellon::load_excellon, gerber::load_gerber},
        toolpath::ToolpathGenerator,
    },
    ui::{
        CopperCrabApp,
        project::{Operation, SourceFile, SourceKind},
        widget::labels::title_label,
    },
};

#[derive(Debug)]
pub(crate) enum FileTreeActions {
    DeleteItem(FileOperationId),
    AddOperation(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum FileOperationId {
    Root,
    File(usize),
    Operation(usize, usize),
}

impl CopperCrabApp {
    pub(crate) fn ui_left(&mut self, ui: &mut egui::Ui) {
        let mut context_menu_actions = Vec::<FileTreeActions>::new();
        title_label(ui, &t!("ui.label.title_files_operations"), false);

        self.add_file_buttons(ui);

        if ui
            .add(Button::new(t!("ui.button.tool_library")).full_width())
            .clicked()
        {
            self.tools.tool_library_open = true;
        }

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.set_height(ui.available_height());

            ui.add_space(8.0);

            if ui
                .add(
                    Button::new(t!("ui.button.export_files"))
                        .accent(Accent::Green)
                        .full_width(),
                )
                .clicked()
            {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    info!("Start export to: '{}'", folder.to_string_lossy());

                    for file in self.files.iter() {
                        info!("Export '{}'", file.name);
                        file.export_gcode(
                            &folder,
                            &self.tools.tool_library,
                            &self.app_config.common_parameters,
                        );
                    }
                }
            }

            ui.vertical(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    TreeView::new(Id::new("files_operation_tree"))
                        .override_striped(Some(true))
                        .allow_drag_and_drop(false)
                        .show_state(ui, &mut self.tree_files_states, |builder| {
                            builder.dir(FileOperationId::Root, t!("ui.label.project"));

                            for (id, file) in self.files.iter_mut().enumerate() {
                                Self::build_tree_file_node(
                                    builder,
                                    id,
                                    file,
                                    &mut context_menu_actions,
                                );
                            }

                            builder.close_dir();
                        });
                });
            });
        });

        for action in context_menu_actions {
            self.handle_action(action);
        }
    }

    fn add_file_buttons(&mut self, ui: &mut Ui) {
        ui.columns(3, |cols| {
            if cols[0]
                .add(
                    elegance::Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS,
                        t!("ui.button.copper")
                    ))
                    .full_width(),
                )
                .clicked()
            {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Copper Gerber Files", &["gbr", "ger", "gtl", "gbl"])
                    .pick_files()
                {
                    for path in files {
                        match load_gerber(&path) {
                            Ok(layer) => {
                                match ToolpathGenerator::new(&layer, false) {
                                    Ok(generator) => {
                                        self.files.push(SourceFile::new(
                                            path.file_stem()
                                                .and_then(|f| f.to_str())
                                                .unwrap_or("Copper")
                                                .into(),
                                            SourceKind::Copper(generator),
                                        ));
                                        self.center_view_to_board();
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "{}",
                                            t!(
                                                "ui.toolpath.error.fail_generate",
                                                e = e.to_string()
                                            )
                                        );
                                    }
                                };
                            }
                            Err(e) => {
                                log::error!(
                                    "{}",
                                    t!("ui.toolpath.error.fail_load", e = e.to_string())
                                )
                            }
                        }
                    }
                }
            }

            if cols[1]
                .add(
                    elegance::Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS,
                        t!("ui.button.outline")
                    ))
                    .full_width(),
                )
                .clicked()
            {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Outline Gerber Files", &["gbr", "ger", "gto", "gko"])
                    .pick_files()
                {
                    for path in files {
                        match load_gerber(&path) {
                            Ok(layer) => {
                                match ToolpathGenerator::new(&layer, true) {
                                    Ok(generator) => {
                                        self.files.push(SourceFile::new(
                                            path.file_stem()
                                                .and_then(|f| f.to_str())
                                                .unwrap_or("Outline")
                                                .into(),
                                            SourceKind::Outline(generator),
                                        ));
                                        self.center_view_to_board();
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "{}",
                                            t!(
                                                "ui.toolpath.error.fail_generate",
                                                e = e.to_string()
                                            )
                                        );
                                    }
                                };
                            }
                            Err(e) => {
                                log::error!(
                                    "{}",
                                    t!("ui.toolpath.error.fail_load", e = e.to_string())
                                )
                            }
                        }
                    }
                }
            }

            if cols[2]
                .add(
                    elegance::Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS,
                        t!("ui.button.drill")
                    ))
                    .full_width(),
                )
                .clicked()
            {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Drill Files", &["drl", "txt"])
                    .pick_files()
                {
                    for path in files {
                        match load_excellon(&path) {
                            Ok(layer) => {
                                self.files.push(SourceFile::new(
                                    path.file_stem()
                                        .and_then(|f| f.to_str())
                                        .unwrap_or("Drill")
                                        .into(),
                                    SourceKind::Drill(layer),
                                ));
                                self.center_view_to_board();
                            }
                            Err(e) => {
                                log::error!(
                                    "{}",
                                    t!("ui.excellon.error.fail_load", e = e.to_string())
                                )
                            }
                        }
                    }
                }
            }
        });
    }

    fn build_tree_file_node(
        builder: &mut TreeViewBuilder<'_, FileOperationId>,
        file_id: usize,
        file: &mut SourceFile,
        actions: &mut Vec<FileTreeActions>,
    ) {
        let icon = file.source.icon();
        let add_label = file.source.add_operation_label();

        let id = FileOperationId::File(file_id);
        let node = NodeBuilder::dir(id.clone())
            .label_ui(|ui| {
                ui.add(egui::Label::new(format!("{} {}", icon, &file.name)).selectable(false));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(Switch::new(&mut file.is_visible, ""))
                });
            })
            .context_menu(|ui| {
                if ui.button(&add_label).clicked() {
                    actions.push(FileTreeActions::AddOperation(file_id));
                    ui.close();
                }
                ui.separator();
                if ui.button(t!("ui.button.delete_file")).clicked() {
                    actions.push(FileTreeActions::DeleteItem(id.clone()));
                    ui.close();
                }
            });

        builder.node(node);

        for (i, op) in file.operations.iter().enumerate() {
            builder.leaf(FileOperationId::Operation(file_id, i), op.name.clone());
        }

        builder.close_dir();
    }

    pub(crate) fn handle_action(&mut self, action: FileTreeActions) {
        match action {
            FileTreeActions::DeleteItem(file_operation_id) => {
                match file_operation_id {
                    FileOperationId::Root => (),
                    FileOperationId::File(id) => {
                        self.files.remove(id);
                    }
                    FileOperationId::Operation(id_file, id_op) => {
                        if let Some(file) = self.files.get_mut(id_file) {
                            file.operations.remove(id_op);
                        }
                    }
                };
            }

            FileTreeActions::AddOperation(file_id) => {
                if let Some(file) = self.files.get_mut(file_id) {
                    let name = file.next_operation_name();
                    match file.source {
                        SourceKind::Copper(_) => {
                            file.operations.push(Operation::new_isolation(name));
                        }
                        SourceKind::Outline(_) => {
                            file.operations.push(Operation::new_outline(name));
                        }
                        SourceKind::Drill(_) => {
                            file.operations.push(Operation::new_drill(name));
                        }
                    }
                }
            }
        };
    }
}
