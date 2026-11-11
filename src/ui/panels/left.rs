use egui::{Align, Id, Layout, Ui};
use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder};
use elegance::Switch;
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
    },
};

#[derive(Debug)]
pub(crate) enum FileTreeActions {
    DeleteItem(FileOperationId),
    AddOperation(usize),
    ToggleVisibility(usize),
    RestorePrimitive(usize, (usize, usize)),
    ExportFile(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum FileOperationId {
    Root,
    File(usize),
    Hidden(usize),
    Operation(usize, usize),
}

impl CopperCrabApp {
    pub(crate) fn ui_left(&mut self, ui: &mut egui::Ui) {
        let mut context_menu_actions = Vec::<FileTreeActions>::new();

        self.add_file_buttons(ui);

        let buttons_height = ui.spacing().interact_size.y + 16.0;

        let available_height = ui.available_height() - buttons_height;

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), available_height),
            Layout::top_down(Align::Min),
            |ui| {
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
            },
        );

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let btn_h = ui.spacing().interact_size.y;

            if ui
                .add_sized(
                    [btn_h, btn_h],
                    egui::Button::new(egui_phosphor::regular::GEAR),
                )
                .on_hover_text(t!("ui.button.export_settings"))
                .clicked()
            {
                self.export_settings_open = true;
            }

            let remaining = ui.available_width();
            if ui
                .add_sized(
                    [remaining, btn_h],
                    egui::Button::new(t!("ui.button.export_files")),
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
                            &self.export_settings,
                        );
                    }
                }
            }
        });
        ui.add_space(28.0);

        for action in context_menu_actions {
            self.handle_action(action);
        }
    }

    fn add_file_buttons(&mut self, ui: &mut Ui) {
        let btn_h = ui.spacing().interact_size.y;

        ui.vertical(|ui| {
            if ui
                .add_sized(
                    [ui.available_width(), btn_h],
                    egui::Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS,
                        t!("ui.button.copper")
                    )),
                )
                .clicked()
            {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Copper Gerber Files", &["gbr", "ger", "gtl", "gbl"])
                    .pick_files()
                {
                    for path in files {
                        match load_gerber(&path) {
                            Ok(layer) => match ToolpathGenerator::new(&layer, false) {
                                Ok(generator) => {
                                    self.files.push(SourceFile::new(
                                        path.file_stem()
                                            .and_then(|f| f.to_str())
                                            .unwrap_or("Copper")
                                            .into(),
                                        SourceKind::Copper { generator, layer },
                                        Some(path),
                                    ));
                                    self.center_view_to_board();
                                }
                                Err(e) => log::error!(
                                    "{}",
                                    t!("ui.toolpath.error.fail_generate", e = e.to_string())
                                ),
                            },
                            Err(e) => log::error!(
                                "{}",
                                t!("ui.toolpath.error.fail_load", e = e.to_string())
                            ),
                        }
                    }
                }
            }

            if ui
                .add_sized(
                    [ui.available_width(), btn_h],
                    egui::Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS,
                        t!("ui.button.outline")
                    )),
                )
                .clicked()
            {
                if let Some(files) = rfd::FileDialog::new()
                    .add_filter("Outline Gerber Files", &["gbr", "ger", "gto", "gko"])
                    .pick_files()
                {
                    for path in files {
                        match load_gerber(&path) {
                            Ok(layer) => match ToolpathGenerator::new(&layer, true) {
                                Ok(generator) => {
                                    self.files.push(SourceFile::new(
                                        path.file_stem()
                                            .and_then(|f| f.to_str())
                                            .unwrap_or("Outline")
                                            .into(),
                                        SourceKind::Outline(generator),
                                        Some(path),
                                    ));
                                    self.center_view_to_board();
                                }
                                Err(e) => log::error!(
                                    "{}",
                                    t!("ui.toolpath.error.fail_generate", e = e.to_string())
                                ),
                            },
                            Err(e) => log::error!(
                                "{}",
                                t!("ui.toolpath.error.fail_load", e = e.to_string())
                            ),
                        }
                    }
                }
            }

            if ui
                .add_sized(
                    [ui.available_width(), btn_h],
                    egui::Button::new(format!(
                        "{} {}",
                        egui_phosphor::regular::PLUS,
                        t!("ui.button.drill")
                    )),
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
                                    Some(path),
                                ));
                                self.center_view_to_board();
                            }
                            Err(e) => log::error!(
                                "{}",
                                t!("ui.excellon.error.fail_load", e = e.to_string())
                            ),
                        }
                    }
                }
            }

            ui.add_space(4.0);

            if ui
                .add_sized(
                    [ui.available_width(), btn_h],
                    egui::Button::new(t!("ui.button.tool_library")),
                )
                .clicked()
            {
                self.tools.tool_library_open = true;
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
        let is_visible = file.is_visible;
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
                let visibility_text = if is_visible {
                    format!(
                        "{} {}",
                        egui_phosphor::regular::EYE_SLASH,
                        t!("ui.button.hide")
                    )
                } else {
                    format!("{} {}", egui_phosphor::regular::EYE, t!("ui.button.show"))
                };
                if ui.button(visibility_text).clicked() {
                    actions.push(FileTreeActions::ToggleVisibility(file_id));
                    ui.close();
                }
                ui.separator();
                if ui
                    .button(format!(
                        "{} {}",
                        egui_phosphor::regular::EXPORT,
                        t!("ui.button.export_file")
                    ))
                    .clicked()
                {
                    actions.push(FileTreeActions::ExportFile(file_id));
                    ui.close();
                }
                ui.separator();
                if ui.button(t!("ui.button.delete_file")).clicked() {
                    actions.push(FileTreeActions::DeleteItem(id.clone()));
                    ui.close();
                }
            });

        builder.node(node);

        if let SourceKind::Copper { .. } = &file.source {
            if !file.hidden_primitives.is_empty() {
                let hidden_id = FileOperationId::Hidden(file_id);
                let hidden_node = NodeBuilder::dir(hidden_id).label(format!(
                    "{} {} ({})",
                    egui_phosphor::regular::EYE_SLASH,
                    t!("ui.label.hidden_primitives"),
                    file.hidden_primitives.len()
                ));
                builder.node(hidden_node);

                for &(t_idx, p_idx) in &file.hidden_primitives {
                    let restore_id =
                        FileOperationId::Operation(file_id, 999_999 + t_idx * 10_000 + p_idx);
                    let restore_leaf = NodeBuilder::leaf(restore_id)
                        .label(format!("{} #{}", t!("ui.label.object"), p_idx))
                        .context_menu(|ui| {
                            if ui
                                .button(format!(
                                    "{} {}",
                                    egui_phosphor::regular::ARROW_U_UP_LEFT,
                                    t!("ui.button.restore")
                                ))
                                .clicked()
                            {
                                actions.push(FileTreeActions::RestorePrimitive(
                                    file_id,
                                    (t_idx, p_idx),
                                ));
                                ui.close();
                            }
                        });
                    builder.node(restore_leaf);
                }
                builder.close_dir();
            }
        }

        for (i, op) in file.operations.iter().enumerate() {
            let op_id = FileOperationId::Operation(file_id, i);

            let leaf = NodeBuilder::leaf(op_id.clone())
                .label(op.name.clone())
                .context_menu(|ui| {
                    if ui
                        .button(format!(
                            "{} {}",
                            egui_phosphor::regular::TRASH,
                            t!("ui.button.delete_operation")
                        ))
                        .clicked()
                    {
                        actions.push(FileTreeActions::DeleteItem(op_id.clone()));
                        ui.close();
                    }
                });

            builder.node(leaf);
        }

        builder.close_dir();
    }

    pub(crate) fn handle_action(&mut self, action: FileTreeActions) {
        match action {
            FileTreeActions::DeleteItem(file_operation_id) => match file_operation_id {
                FileOperationId::Root => (),
                FileOperationId::Hidden(_) => (),
                FileOperationId::File(id) => {
                    self.files.remove(id);
                    self.tree_files_states = Default::default();
                }
                FileOperationId::Operation(id_file, id_op) => {
                    if id_op < 999_999 {
                        if let Some(file) = self.files.get_mut(id_file) {
                            file.operations.remove(id_op);
                        }
                        self.tree_files_states = Default::default();
                    }
                }
            },
            FileTreeActions::AddOperation(file_id) => {
                if let Some(file) = self.files.get_mut(file_id) {
                    let name = file.next_operation_name();
                    match file.source {
                        SourceKind::Copper { .. } => {
                            file.operations.push(Operation::new_isolation(name))
                        }
                        SourceKind::Outline(_) => {
                            file.operations.push(Operation::new_outline(name))
                        }
                        SourceKind::Drill(_) => file.operations.push(Operation::new_drill(name)),
                    }
                }
            }
            FileTreeActions::ToggleVisibility(file_id) => {
                if let Some(file) = self.files.get_mut(file_id) {
                    file.is_visible = !file.is_visible;
                }
            }
            FileTreeActions::RestorePrimitive(file_id, key) => {
                if let Some(file) = self.files.get_mut(file_id) {
                    file.hidden_primitives.remove(&key);
                    self.generate_file_toolpaths(file_id);
                }
            }
            FileTreeActions::ExportFile(file_id) => {
                self.export_single_file(file_id);
            }
        };
    }
}
