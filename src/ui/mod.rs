use std::{
    collections::VecDeque,
    fmt::Debug,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use egui_ltreeview::{NodeBuilder, TreeView, TreeViewBuilder, TreeViewState};
use log::info;
use rust_i18n::t;

use clipper2::Paths;
use egui::{Align, Color32, DragValue, Id, Layout, Response, Ui, Vec2, WidgetText};
use elegance::{Accent, Button, Select, Switch, TextInput, Theme};
use sanitize_filename::Options;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        DrillLayer,
        bounding_box::BoundingBox,
        geometry::Point2d,
        parser::{excellon::load_excellon, gerber::load_gerber},
        toolpath::{ToolpathGenerator, generate_drill_gcode, generate_isolation_gcode},
        tools::{CncTool, library::ToolLibrary},
    },
    logger::LogEntry,
    ui::{
        app_config::AppConfig,
        colors::*,
        log_ui::panel_logs,
        pcb_transform::PcbTransform,
        widget::{
            canvas::{draw_axes, draw_paths, draw_paths_stroke},
            labels::title_label,
            status_bar::StatusBar,
            tool_library::ToolLibraryUi,
        },
    },
};

pub mod app_config;
mod colors;
mod log_ui;
mod pcb_transform;
mod widget;

const TOOLS_FILENAME: &str = "tools.toml";
const AVAILABLE_LOCALE: [(&str, &str); 2] = [("en", "English"), ("fr", "Français")];

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
enum SelectedTool {
    VBit(usize),
    EndMill(usize),
    Drill(usize),

    #[default]
    None,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ContextTools {
    tool_library_open: bool,
    tool_library: ToolLibrary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct CommonParameters {
    z_safe: f64,
    z_finish: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CanvasState {
    zoom: f32,
    offset: egui::Vec2,
    rect: egui::Rect,
}

#[derive(Debug)]
pub struct CopperCrabApp {
    files: Vec<SourceFile>,
    tree_files_states: TreeViewState<FileOperationId>,

    tools: ContextTools,
    common_parameters: CommonParameters,
    canvas: CanvasState,
    tool_library_ui: ToolLibraryUi,

    status_bar: StatusBar,

    log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,

    app_config: AppConfig,
}

impl Default for CommonParameters {
    fn default() -> Self {
        Self {
            z_safe: 3.0,
            z_finish: 5.0,
        }
    }
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
            rect: egui::Rect {
                min: Default::default(),
                max: Default::default(),
            },
        }
    }
}

#[derive(Debug)]
enum SourceKind {
    Copper(ToolpathGenerator),
    Outline(ToolpathGenerator),
    Drill(DrillLayer),
}

#[derive(Debug)]
enum OperationKind {
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

#[derive(Debug)]
struct Operation {
    name: String,
    tool: SelectedTool,
    depth: f64,
    toolpath: Vec<Paths>,
    kind: OperationKind,
}

impl Operation {
    fn new_isolation(name: String) -> Self {
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

    fn new_outline(name: String) -> Self {
        Self {
            name,
            tool: SelectedTool::None,
            depth: 0.0,
            toolpath: Vec::new(),
            kind: OperationKind::Outline,
        }
    }

    fn new_drill(name: String) -> Self {
        Self {
            name,
            tool: SelectedTool::None,
            depth: 0.0,
            toolpath: Vec::new(),
            kind: OperationKind::Drill { peck_step: 1.0 },
        }
    }
}

impl SourceKind {
    fn icon(&self) -> &'static str {
        match self {
            SourceKind::Copper(_) => egui_phosphor::regular::GIT_MERGE,
            SourceKind::Outline(_) => egui_phosphor::regular::SQUARE,
            SourceKind::Drill(_) => egui_phosphor::regular::CIRCLES_THREE_PLUS,
        }
    }

    fn operation_prefix(&self) -> &'static str {
        match self {
            SourceKind::Copper(_) => "Isolation",
            SourceKind::Outline(_) => "Outline",
            SourceKind::Drill(_) => "Drill",
        }
    }

    fn add_operation_label(&self) -> String {
        match self {
            SourceKind::Copper(_) => t!("ui.button.add_isolation_operation").to_string(),
            SourceKind::Outline(_) => t!("ui.button.add_outline_operation").to_string(),
            SourceKind::Drill(_) => t!("ui.button.add_drill_operation").to_string(),
        }
    }

    fn as_copper(&self) -> Option<&ToolpathGenerator> {
        match self {
            SourceKind::Copper(g) => Some(g),
            _ => None,
        }
    }

    fn as_outline(&self) -> Option<&ToolpathGenerator> {
        match self {
            SourceKind::Outline(g) => Some(g),
            _ => None,
        }
    }

    fn as_drill(&self) -> Option<&DrillLayer> {
        match self {
            SourceKind::Drill(d) => Some(d),
            _ => None,
        }
    }

    fn mirror_x(&mut self, center_x: Option<f64>) {
        match self {
            SourceKind::Copper(toolpath_generator) | SourceKind::Outline(toolpath_generator) => {
                toolpath_generator.mirror_x(center_x)
            }
            SourceKind::Drill(drill_layer) => drill_layer.mirror_x(center_x),
        }
    }

    fn mirror_y(&mut self, center_y: Option<f64>) {
        match self {
            SourceKind::Copper(toolpath_generator) | SourceKind::Outline(toolpath_generator) => {
                toolpath_generator.mirror_y(center_y)
            }
            SourceKind::Drill(drill_layer) => drill_layer.mirror_y(center_y),
        }
    }

    fn translate(&mut self, offset: Point2d) {
        match self {
            SourceKind::Copper(toolpath_generator) | SourceKind::Outline(toolpath_generator) => {
                toolpath_generator.translate(offset)
            }
            SourceKind::Drill(drill_layer) => drill_layer.translate(offset),
        }
    }
}

impl OperationKind {
    fn title(&self) -> String {
        match self {
            OperationKind::Isolation { .. } => {
                t!("ui.label.parameters.isolation.title").to_string()
            }
            OperationKind::Outline => t!("ui.label.parameters.outline.title").to_string(),
            OperationKind::Drill { .. } => t!("ui.label.parameters.drill.title").to_string(),
        }
    }

    fn tool_select_label(&self) -> String {
        match self {
            OperationKind::Isolation { .. } => t!("ui.select_tool.isolation").to_string(),
            OperationKind::Outline => t!("ui.select_tool.outline").to_string(),
            OperationKind::Drill { .. } => t!("ui.select_tool.drill").to_string(),
        }
    }

    fn depth_texts(&self) -> (String, String) {
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

    fn depth_range(&self) -> std::ops::RangeInclusive<f64> {
        match self {
            OperationKind::Isolation { .. } => 0.01..=5.0,
            _ => 0.1..=20.0,
        }
    }
}

#[derive(Debug)]
struct SourceFile {
    pub name: String,
    pub is_visible: bool,
    pub source: SourceKind,
    pub operations: Vec<Operation>,
}

impl SourceFile {
    fn next_operation_name(&self) -> String {
        format!(
            "{} #{}",
            self.source.operation_prefix(),
            self.operations.len() + 1
        )
    }

    fn export_gcode(
        &self,
        folder: &PathBuf,
        tool_library: &ToolLibrary,
        common: &CommonParameters,
    ) {
        for op in self.operations.iter() {
            match op.kind {
                OperationKind::Isolation {
                    passes: _,
                    overlap: _,
                    offset: _,
                } => {
                    let tool = match op.tool {
                        SelectedTool::VBit(id) => {
                            tool_library.vbits.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::EndMill(id) => {
                            tool_library.end_mills.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::Drill(_) | SelectedTool::None => None,
                    };

                    if tool.is_none() {
                        log::error!("{}", t!("ui.error.bad_isolation_tool"));
                        return;
                    }

                    let gcode = generate_isolation_gcode(
                        &op.toolpath,
                        tool.unwrap(),
                        op.depth,
                        common.z_safe,
                        common.z_finish,
                    );

                    let gcode_file = folder.join(sanitize_filename::sanitize_with_options(
                        &op.name,
                        Options {
                            replacement: "_",
                            ..Default::default()
                        },
                    ));

                    if let Err(e) = std::fs::write(&gcode_file, gcode) {
                        log::error!("{}", t!("ui.error.fail_write_isolation", e = e.to_string()))
                    }
                }

                OperationKind::Outline => {
                    let tool = match op.tool {
                        SelectedTool::EndMill(id) => {
                            tool_library.end_mills.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::VBit(_) | SelectedTool::Drill(_) | SelectedTool::None => None,
                    };

                    if tool.is_none() {
                        log::error!("{}", t!("ui.error.bad_outline_tool"));
                        return;
                    }

                    let gcode = generate_isolation_gcode(
                        &op.toolpath,
                        tool.unwrap(),
                        op.depth,
                        common.z_safe,
                        common.z_finish,
                    );

                    let gcode_file = folder.join(sanitize_filename::sanitize_with_options(
                        &op.name,
                        Options {
                            replacement: "_",
                            ..Default::default()
                        },
                    ));

                    if let Err(e) = std::fs::write(&gcode_file, gcode) {
                        log::error!("{}", t!("ui.error.fail_write_outline", e = e.to_string()))
                    }
                }

                OperationKind::Drill { peck_step } => {
                    let tool = match op.tool {
                        SelectedTool::Drill(id) => {
                            tool_library.drill_bits.get(id).map(|t| t as &dyn CncTool)
                        }
                        SelectedTool::VBit(_) | SelectedTool::EndMill(_) | SelectedTool::None => {
                            None
                        }
                    };

                    if tool.is_none() {
                        log::error!("{}", t!("ui.error.bad_drill_tool"));
                        return;
                    }

                    let gcode = generate_drill_gcode(
                        self.source.as_drill().as_ref().unwrap(),
                        tool.unwrap(),
                        op.depth,
                        common.z_safe,
                        common.z_finish,
                        peck_step,
                    );

                    let gcode_file = folder.join(sanitize_filename::sanitize_with_options(
                        &op.name,
                        Options {
                            replacement: "_",
                            ..Default::default()
                        },
                    ));

                    if let Err(e) = std::fs::write(&gcode_file, gcode) {
                        log::error!("{}", t!("ui.error.fail_write_drill", e = e.to_string()))
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum FileTreeActions {
    DeleteItem(FileOperationId),
    AddOperation(usize, String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FileOperationId {
    Root,
    File(usize),
    Operation(usize, usize),
}

impl CopperCrabApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
        app_config: AppConfig,
        config_folder: PathBuf,
    ) -> Self {
        let mut tools_file = PathBuf::from(&config_folder);
        tools_file.push(TOOLS_FILENAME);

        cc.egui_ctx.global_style_mut(|style| {
            style.spacing.button_padding = egui::vec2(10.0, 4.0);
        });

        Self {
            files: Default::default(),
            tree_files_states: Default::default(),
            tools: ContextTools {
                tool_library: ToolLibrary::new(&config_folder),
                ..Default::default()
            },
            common_parameters: app_config.common_parameters.clone(),
            canvas: Default::default(),
            tool_library_ui: Default::default(),
            status_bar: Default::default(),
            log_buffer: log_buffer,
            app_config,
        }
    }

    fn ui_left(&mut self, ui: &mut egui::Ui) {
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
                            &self.common_parameters,
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
                                        self.files.push(SourceFile {
                                            name: path
                                                .file_stem()
                                                .and_then(|f| f.to_str())
                                                .unwrap_or("Copper")
                                                .into(),
                                            is_visible: true,
                                            source: SourceKind::Copper(generator),
                                            operations: Default::default(),
                                        });
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
                                        self.files.push(SourceFile {
                                            name: path
                                                .file_stem()
                                                .and_then(|f| f.to_str())
                                                .unwrap_or("Outline")
                                                .into(),
                                            is_visible: true,
                                            source: SourceKind::Outline(generator),
                                            operations: Default::default(),
                                        });
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
                                self.files.push(SourceFile {
                                    name: path
                                        .file_stem()
                                        .and_then(|f| f.to_str())
                                        .unwrap_or("Drill")
                                        .into(),
                                    is_visible: true,
                                    source: SourceKind::Drill(layer),
                                    operations: Default::default(),
                                });
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
        let next_name = file.next_operation_name();

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
                    actions.push(FileTreeActions::AddOperation(file_id, next_name.clone()));
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

    fn ui_right(&mut self, ui: &mut egui::Ui) {
        let mut action: Option<FileTreeActions> = None;
        let theme = Theme::current(ui.ctx());

        let mut has_changed = false;

        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(selected) = self.tree_files_states.selected().first() {
                match *selected {
                    FileOperationId::Root => {
                        title_label(ui, &t!("ui.label.project"), false);

                        dragvalue_with_label(
                            ui,
                            DragValue::new(&mut self.common_parameters.z_safe)
                                .range(0.1..=10.0)
                                .speed(0.1)
                                .suffix(" mm"),
                            t!("ui.label.parameters.common.safe_z"),
                            t!("ui.tooltip.parameters.common.safe_z"),
                        );

                        ui.add_space(8.0);

                        dragvalue_with_label(
                            ui,
                            DragValue::new(&mut self.common_parameters.z_finish)
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
                    FileOperationId::File(id) => {
                        title_label(ui, &t!("ui.label.file"), false);
                        if let Some(file) = self.files.get_mut(id) {
                            let add_op_text = file.source.add_operation_label();
                            let next_op_name = file.next_operation_name();

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
                                action = Some(FileTreeActions::AddOperation(id, next_op_name));
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
                                action =
                                    Some(FileTreeActions::DeleteItem(FileOperationId::File(id)));
                            }

                            if has_changed {
                                self.generate_file_toolpaths(id);
                            }
                        }
                    }

                    FileOperationId::Operation(id_file, id_op) => {
                        if let Some(operation) = self
                            .files
                            .get_mut(id_file)
                            .and_then(|f| f.operations.get_mut(id_op))
                        {
                            let lib = &self.tools.tool_library;

                            title_label(ui, &operation.kind.title(), false);

                            ui.add(
                                TextInput::new(&mut operation.name)
                                    .label(t!("ui.label.operation.name")),
                            );
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
                                    let iso_width = Self::tool_cutting_width(
                                        lib,
                                        &operation.tool,
                                        operation.depth,
                                    );

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
                                        DragValue::new(overlap)
                                            .range(0..=100)
                                            .speed(1)
                                            .suffix(" %"),
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
                                action = Some(FileTreeActions::DeleteItem(
                                    FileOperationId::Operation(id_file, id_op),
                                ));
                            }
                        }

                        if has_changed {
                            self.generate_operation_toolpaths(id_file, id_op);
                        }
                    }
                }
            };

            if let Some(action) = action {
                self.handle_action(action);
            }
        });
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

    fn ui_canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        let rect = response.rect;

        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(13, 26, 13));

        let transform = PcbTransform::new(rect, self.canvas.zoom, self.canvas.offset);

        draw_axes(&painter, &transform);

        let (sel_file_id, sel_op_id) = self
            .tree_files_states
            .selected()
            .first()
            .and_then(|f| match f {
                FileOperationId::Operation(file, op) => Some((*file, *op)),
                _ => None,
            })
            .unwrap_or((usize::MAX, usize::MAX));

        for (file_id, file) in self.files.iter().enumerate() {
            if file.is_visible == false {
                continue;
            }

            match &file.source {
                SourceKind::Copper(copper) => {
                    draw_paths(&painter, &copper.traces, &transform, COPPER, 1.0);

                    for (op_id, operation) in file.operations.iter().enumerate() {
                        let color = if file_id == sel_file_id && op_id == sel_op_id {
                            SELECTED_PATH
                        } else {
                            COPPER_PATH
                        };

                        for paths in &operation.toolpath {
                            draw_paths_stroke(&painter, paths, &transform, color, 1.0);
                        }
                    }
                }

                SourceKind::Outline(outline) => {
                    draw_paths_stroke(&painter, &outline.traces, &transform, OUTLINE, 1.0);

                    for operation in &file.operations {
                        for paths in &operation.toolpath {
                            draw_paths_stroke(&painter, paths, &transform, OUTLINE_PATH, 1.0);
                        }
                    }
                }

                SourceKind::Drill(drill) => {
                    for hole in &drill.holes {
                        painter.circle_filled(
                            transform.to_screen(hole.x, hole.y),
                            (hole.diameter / 2.0) as f32 * self.canvas.zoom,
                            Color32::WHITE,
                        );
                    }
                }
            }
        }

        // zoom/pan
        if response.dragged() {
            self.canvas.offset += response.drag_delta();
        }

        if let Some(pos) = response.hover_pos() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom_factor = 1.0 + scroll * 0.001;
                let cursor_in_canvas = pos - rect.center();

                self.zoom_on_position(zoom_factor, cursor_in_canvas);
            }

            let pcb_x = (pos.x - transform.rect.min.x - transform.offset.x) / transform.scale;
            let pcb_y = -(pos.y - transform.rect.min.y - transform.offset.y) / transform.scale;
            self.status_bar.set_mouse_pcb(pcb_x, pcb_y);
            self.canvas.rect = rect;
        }

        egui::Area::new(egui::Id::new("canvas_controls"))
            .fixed_pos(egui::pos2(rect.max.x - 110.0, rect.max.y - 40.0))
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(Button::icon('−', "").accent(Accent::Purple))
                        .clicked()
                    {
                        self.zoom_on_position(0.95, Vec2::ZERO);
                    }
                    if ui
                        .add(Button::icon('+', "").accent(Accent::Purple))
                        .clicked()
                    {
                        self.zoom_on_position(1.05, Vec2::ZERO);
                    }
                    if ui
                        .add(Button::icon('⊡', "").accent(Accent::Purple))
                        .clicked()
                    {
                        // Reset zoom et offset
                        self.center_view_to_board();
                    }
                });
            });
    }

    fn zoom_on_position(&mut self, zoom_factor: f32, position: Vec2) {
        let old_zoom = self.canvas.zoom;
        self.canvas.zoom *= zoom_factor;

        self.canvas.offset =
            position + (self.canvas.offset - position) * (self.canvas.zoom / old_zoom);
    }

    fn get_bounding_box(&self) -> BoundingBox {
        let mut b_box = BoundingBox::new();

        for file in &self.files {
            match &file.source {
                SourceKind::Copper(toolpath_generator) => {
                    b_box.update_from_bounds(&toolpath_generator.traces.bounds())
                }
                SourceKind::Outline(toolpath_generator) => {
                    b_box.update_from_bounds(&toolpath_generator.traces.bounds())
                }
                SourceKind::Drill(drill_layer) => {
                    for p in drill_layer.holes.iter() {
                        b_box.update_from_point(p.x, p.y);
                    }
                }
            }

            for op in &file.operations {
                for p in &op.toolpath {
                    b_box.update_from_bounds(&p.bounds());
                }
            }
        }

        b_box
    }

    fn center_view_to_board(&mut self) {
        let b_box = self.get_bounding_box();

        if b_box.is_infinite() {
            self.canvas.offset.x = 0.0;
            self.canvas.offset.y = 0.0;
            self.canvas.zoom = 1.0;
        } else {
            let (min_x, min_y, max_x, max_y) = b_box.to_tuples();
            let pcb_w = (max_x - min_x) as f32;
            let pcb_h = (max_y - min_y) as f32;

            if pcb_w > 0.0 && pcb_h > 0.0 {
                let scale_x = self.canvas.rect.width() * 0.9 / pcb_w;
                let scale_y = self.canvas.rect.height() * 0.9 / pcb_h;
                self.canvas.zoom = scale_x.min(scale_y);
            } else {
                self.canvas.zoom = 1.0;
            }

            let pcb_cx = ((min_x + max_x) / 2.0) as f32;
            let pcb_cy = ((min_y + max_y) / 2.0) as f32;

            self.canvas.offset.x = -pcb_cx * self.canvas.zoom;
            self.canvas.offset.y = pcb_cy * self.canvas.zoom;
        }
    }

    fn handle_action(&mut self, action: FileTreeActions) {
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

            FileTreeActions::AddOperation(file_id, name) => {
                if let Some(file) = self.files.get_mut(file_id) {
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

    fn generate_file_toolpaths(&mut self, id_file: usize) {
        let nb_ops = self.files.get(id_file).map_or(0, |f| f.operations.len());

        for id_op in 0..nb_ops {
            self.generate_operation_toolpaths(id_file, id_op);
        }
    }

    fn generate_operation_toolpaths(&mut self, id_file: usize, id_op: usize) {
        if let Some(file) = self.files.get_mut(id_file) {
            if let Some(op) = file.operations.get_mut(id_op) {
                match op.kind {
                    OperationKind::Isolation {
                        passes,
                        overlap,
                        offset,
                    } => {
                        let tool: Option<&dyn CncTool> = match op.tool {
                            SelectedTool::VBit(id) => {
                                Some(self.tools.tool_library.vbits.get(id).unwrap())
                            }
                            SelectedTool::EndMill(id) => {
                                Some(self.tools.tool_library.end_mills.get(id).unwrap())
                            }
                            SelectedTool::Drill(_) | SelectedTool::None => None,
                        };

                        if let Some(tool) = tool {
                            op.toolpath = file.source.as_copper().unwrap().generate_toolpaths(
                                tool,
                                op.depth,
                                passes,
                                overlap / 100.0,
                                offset,
                            )
                        } else {
                            op.toolpath = Vec::new();
                        }
                    }
                    OperationKind::Outline => {
                        if let SelectedTool::EndMill(id) = op.tool {
                            op.toolpath = file.source.as_outline().unwrap().generate_toolpaths(
                                self.tools.tool_library.end_mills.get(id).unwrap(),
                                op.depth,
                                1,
                                1.0,
                                0.0,
                            );
                        } else {
                            op.toolpath = Vec::new();
                        }
                    }
                    OperationKind::Drill { peck_step: _ } => (),
                };
            }
        }
    }
}

impl eframe::App for CopperCrabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        match self.app_config.app_theme {
            app_config::AppTheme::Slate => Theme::slate().install(ui.ctx()),
            app_config::AppTheme::Charcoal => Theme::charcoal().install(ui.ctx()),
            app_config::AppTheme::Frost => Theme::frost().install(ui.ctx()),
            app_config::AppTheme::Paper => Theme::paper().install(ui.ctx()),
        };

        let screen_rect = ui.ctx().viewport_rect();
        self.app_config.window_width = screen_rect.width();
        self.app_config.window_height = screen_rect.height();

        self.status_bar.show(
            ui,
            &mut self.app_config.app_locale,
            &mut self.app_config.app_theme,
        );
        panel_logs(ui, &self.log_buffer);

        egui::Panel::left("left_panel")
            .resizable(false)
            .min_size(250.0)
            .show(ui, |ui| {
                self.ui_left(ui);
            });

        egui::Panel::right("right_panel")
            .resizable(false)
            .min_size(200.0)
            .show(ui, |ui| {
                self.ui_right(ui);
            });

        egui::CentralPanel::default().show(ui, |ui| {
            self.ui_canvas(ui);
        });

        self.tool_library_ui.show(
            ui.ctx(),
            &mut self.tools.tool_library_open,
            &mut self.tools.tool_library,
        );
    }

    fn on_exit(&mut self) {
        self.app_config.save();
    }
}

fn dragvalue_with_label(
    ui: &mut Ui,
    drag_value: DragValue,
    label: impl Into<String>,
    tooltip: impl Into<WidgetText>,
) -> Response {
    let theme = Theme::current(ui.ctx());
    let mut resp = None;

    ui.vertical(|ui| {
        ui.label(theme.muted_text(label));
        resp = Some(ui.add(drag_value).on_hover_text(tooltip));
    });

    resp.unwrap()
}
