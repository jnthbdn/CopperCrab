use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rust_i18n::t;

use clipper2::Paths;
use egui::{RichText, Vec2};
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        DrillLayer,
        toolpath::{ToolpathGenerator, generate_drill_gcode, generate_isolation_gcode},
        tools::{CncTool, library::ToolLibrary},
    },
    logger::LogEntry,
    ui::{
        app_config::AppConfig,
        buttons::{button, button_primary, pick_drill_file, pick_gerber_file, toggle},
        canvas::{draw_axes, draw_paths, draw_paths_stroke},
        colors::*,
        labels::title_label,
        log_ui::panel_logs,
        pcb_transform::PcbTransform,
        status_bar::StatusBar,
        tool_library::ToolLibraryUi,
    },
};

pub mod app_config;
mod buttons;
mod canvas;
mod colors;
mod labels;
mod log_ui;
mod pcb_transform;
mod status_bar;
mod tool_library;

const PICK_TOOL: &str = "Pick a tool";
const TOOLS_FILENAME: &str = "tools.toml";

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
enum SelectedTool {
    VBit(usize),
    EndMill(usize),
    Drill(usize),

    #[default]
    None,
}

#[derive(Debug, Default)]
struct ContextFiles {
    output_folder: Option<PathBuf>,
    gerber_copper_path: Option<PathBuf>,
    gerber_outline_path: Option<PathBuf>,
    drill_path: Option<PathBuf>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ContextTools {
    tool_library_open: bool,
    tool_library: ToolLibrary,

    isolation_tool: SelectedTool,
    outline_tool: SelectedTool,
    drill_tool: SelectedTool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ContextParameters {
    z_safe: f64,
    z_finish: f64,
    isolation_depth: f64,
    isolation_passes: u32,
    isolation_overlap: f64,
    drill_peck_step: f64,
    drill_depth: f64,
    outline_depth: f64,

    export_isolation: bool,
    export_outline: bool,
    export_drill: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ContextLayer {
    show_copper: bool,
    show_outline: bool,
    show_copper_toolpaths: bool,
    show_outline_toolpaths: bool,
    show_drill: bool,

    #[serde(skip)]
    isolation_generator: Option<ToolpathGenerator>,

    #[serde(skip)]
    outline_generator: Option<ToolpathGenerator>,

    #[serde(skip)]
    isolation: Option<Vec<Paths>>,

    #[serde(skip)]
    outline: Option<Vec<Paths>>,

    #[serde(skip)]
    drill: Option<DrillLayer>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CanvasState {
    zoom: f32,
    offset: egui::Vec2,
    rect: egui::Rect,
}

#[derive(Debug)]
pub struct CopperCrabApp {
    files: ContextFiles,
    tools: ContextTools,
    parameters: ContextParameters,
    layers: ContextLayer,

    canvas: CanvasState,
    tool_library_ui: ToolLibraryUi,

    status_bar: StatusBar,

    log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,

    app_config: AppConfig,
}

impl Default for ContextParameters {
    fn default() -> Self {
        Self {
            z_safe: 3.0,
            z_finish: 5.0,
            isolation_depth: 0.1,
            isolation_passes: 2,
            isolation_overlap: 50.0,
            drill_peck_step: 0.5,
            drill_depth: 2.0,
            outline_depth: 2.0,

            export_isolation: true,
            export_outline: true,
            export_drill: false,
        }
    }
}

impl Default for ContextLayer {
    fn default() -> Self {
        Self {
            show_copper: true,
            show_outline: true,
            show_copper_toolpaths: true,
            show_outline_toolpaths: true,
            show_drill: false,

            isolation_generator: None,
            outline_generator: None,
            isolation: None,
            outline: None,
            drill: None,
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
            style.visuals.override_text_color = Some(TEXT_PRIMARY);
            style.spacing.button_padding = egui::vec2(10.0, 4.0);
        });

        Self {
            files: Default::default(),
            tools: ContextTools {
                tool_library: ToolLibrary::new(&config_folder),
                ..Default::default()
            },
            parameters: app_config.context_parameter.clone(),
            layers: app_config.context_layer.clone(),
            canvas: Default::default(),
            tool_library_ui: Default::default(),
            status_bar: Default::default(),
            log_buffer: log_buffer,
            app_config,
        }
    }

    fn ui_left(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.ui_left_file(ui);
            self.ui_left_tools(ui);
            self.ui_left_layers(ui);
        });
    }

    fn ui_left_file(&mut self, ui: &mut egui::Ui) {
        title_label(ui, &t!("ui.files.title"), false);
        ui.separator();

        if pick_gerber_file(
            ui,
            &mut self.files.gerber_copper_path,
            &mut self.layers.isolation_generator,
            &["gbr", "ger", "gtl", "gbl"],
            &t!("ui.files.pick_copper"),
            false,
        ) {
            self.status_bar.set_need_regenerate(true);
            self.center_view_to_board();
        }

        if pick_gerber_file(
            ui,
            &mut self.files.gerber_outline_path,
            &mut self.layers.outline_generator,
            &["gbr", "ger", "gto", "gko"],
            &t!("ui.files.pick_outline"),
            true,
        ) {
            self.status_bar.set_need_regenerate(true);
            self.center_view_to_board();
        }

        pick_drill_file(
            ui,
            &mut self.files.drill_path,
            &mut self.layers.drill,
            &t!("ui.files.pick_drill"),
        );
    }

    fn ui_left_tools(&mut self, ui: &mut egui::Ui) {
        title_label(ui, &t!("ui.select_tool.title"), true);
        ui.separator();

        // Isolation — VBit or EndMill
        ui.label(t!("ui.select_tool.isolation"));
        if egui::ComboBox::from_id_salt("isolation_tool")
            .selected_text(self.isolation_tool_name())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                ui.label(RichText::new(t!("ui.label.library.vbit")).color(TEXT_SECONDARY));
                for (i, vbit) in self.tools.tool_library.vbits.iter().enumerate() {
                    ui.selectable_value(
                        &mut self.tools.isolation_tool,
                        SelectedTool::VBit(i),
                        vbit.to_string(),
                    );
                }

                ui.label(RichText::new(t!("ui.label.library.end_mill")).color(TEXT_SECONDARY));
                for (i, end_mill) in self.tools.tool_library.end_mills.iter().enumerate() {
                    ui.selectable_value(
                        &mut self.tools.isolation_tool,
                        SelectedTool::EndMill(i),
                        end_mill.to_string(),
                    );
                }
            })
            .response
            .changed()
        {
            self.status_bar.set_need_regenerate(true);
        }

        // Outline — EndMill only
        ui.add_space(4.0);
        ui.label(t!("ui.select_tool.outline"));
        if egui::ComboBox::from_id_salt("outline_tool")
            .selected_text(self.outline_tool_name())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (i, end_mill) in self.tools.tool_library.end_mills.iter().enumerate() {
                    ui.selectable_value(
                        &mut self.tools.outline_tool,
                        SelectedTool::EndMill(i),
                        end_mill.to_string(),
                    );
                }
            })
            .response
            .changed()
        {
            self.status_bar.set_need_regenerate(true);
        }

        // Drill — DrillBit only
        ui.add_space(4.0);
        ui.label(t!("ui.select_tool.drill"));
        egui::ComboBox::from_id_salt("drill_tool")
            .selected_text(self.drill_tool_name())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (i, drill_bit) in self.tools.tool_library.drill_bits.iter().enumerate() {
                    ui.selectable_value(
                        &mut self.tools.drill_tool,
                        SelectedTool::Drill(i),
                        drill_bit.to_string(),
                    );
                }
            });

        ui.add_space(16.0);
        if button_primary(ui, &t!("ui.button.tool_library"), true).clicked() {
            self.tools.tool_library_open = true;
        }
    }

    fn ui_left_layers(&mut self, ui: &mut egui::Ui) {
        title_label(ui, &t!("ui.label.layer.title"), true);
        ui.separator();

        Self::layer_row(
            ui,
            &t!("ui.label.layer.copper"),
            COPPER,
            &mut self.layers.show_copper,
        );
        Self::layer_row(
            ui,
            &t!("ui.label.layer.copper_toolpath"),
            SUCCESS,
            &mut self.layers.show_copper_toolpaths,
        );
        Self::layer_row(ui, "Outline", WARNING, &mut self.layers.show_outline);
        Self::layer_row(
            ui,
            &t!("ui.label.layer.outline_toolpath"),
            ACCENT,
            &mut self.layers.show_outline_toolpaths,
        );

        Self::layer_row(
            ui,
            &t!("ui.label.layer.drill"),
            TEXT_PRIMARY,
            &mut self.layers.show_drill,
        );
    }

    fn layer_row(ui: &mut egui::Ui, label: &str, color: egui::Color32, visible: &mut bool) {
        ui.horizontal(|ui| {
            ui.colored_label(color, "■");
            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                toggle(ui, visible);
            });
        });
    }

    fn ui_right(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.ui_right_parameters_common(ui);
            self.ui_right_parameters_isolation(ui);
            self.ui_right_parameters_outline(ui);
            self.ui_right_parameters_drill(ui);

            ui.add_space(16.0);

            ui.add_enabled_ui(
                (self.files.gerber_copper_path.is_some()
                    && self.tools.isolation_tool != SelectedTool::None)
                    || (self.files.gerber_outline_path.is_some()
                        && self.tools.outline_tool != SelectedTool::None),
                |ui| {
                    if button_primary(ui, &t!("ui.button.generate_toolpath").to_string(), true)
                        .clicked()
                    {
                        self.run_toolpath_generation();
                    }
                },
            );

            self.ui_right_export(ui);
        });
    }

    fn ui_right_parameters_common(&mut self, ui: &mut egui::Ui) {
        title_label(
            ui,
            &t!("ui.label.parameters.common.title").to_string(),
            false,
        );
        ui.separator();

        egui::Grid::new("common_parameter_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label(t!("ui.label.parameters.common.safe_z"));
                if ui
                    .add(
                        egui::DragValue::new(&mut self.parameters.z_safe)
                            .range(0..=500)
                            .speed(0.1)
                            .suffix(" mm"),
                    )
                    .on_hover_text(t!("ui.tooltip.parameters.common.safe_z"))
                    .changed()
                {
                    self.status_bar.set_need_regenerate(true);
                }
                ui.end_row();

                ui.label(t!("ui.label.parameters.common.final_z"));
                if ui
                    .add(
                        egui::DragValue::new(&mut self.parameters.z_finish)
                            .range(0..=500)
                            .speed(0.1)
                            .suffix(" mm"),
                    )
                    .on_hover_text(t!("ui.tooltip.parameters.common.final_z"))
                    .changed()
                {
                    self.status_bar.set_need_regenerate(true);
                }
                ui.end_row();
            });
    }

    fn ui_right_parameters_isolation(&mut self, ui: &mut egui::Ui) {
        title_label(
            ui,
            &t!("ui.label.parameters.isolation.title").to_string(),
            true,
        );
        ui.separator();

        let iso_width = match self.tools.isolation_tool {
            SelectedTool::VBit(id) => {
                self.tools.tool_library.vbits[id].cutting_width(self.parameters.isolation_depth)
            }
            SelectedTool::EndMill(id) => {
                self.tools.tool_library.end_mills[id].cutting_width(self.parameters.isolation_depth)
            }
            _ => 0.0,
        };

        egui::Grid::new("isolation_parameter_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label(t!("ui.label.parameters.isolation.depth"));
                if ui
                    .add(
                        egui::DragValue::new(&mut self.parameters.isolation_depth)
                            .range(0.01..=5.0)
                            .speed(0.01)
                            .suffix(" mm"),
                    )
                    .on_hover_text(t!("ui.tooltip.parameters.isolation.depth"))
                    .changed()
                {
                    self.status_bar.set_need_regenerate(true);
                }
                ui.end_row();

                //Cutting width
                ui.label(
                    RichText::new(format!(
                        "\t> {}",
                        t!("ui.label.parameters.isolation.depth_result")
                    ))
                    .color(TEXT_SECONDARY),
                );
                ui.label(RichText::new(format!("{:.2} mm", iso_width)).color(TEXT_SECONDARY));
                ui.end_row();

                ui.label(t!("ui.label.parameters.isolation.passes"));
                if ui
                    .add(
                        egui::DragValue::new(&mut self.parameters.isolation_passes)
                            .range(1..=50)
                            .speed(1),
                    )
                    .on_hover_text(t!("ui.tooltip.parameters.isolation.passes"))
                    .changed()
                {
                    self.status_bar.set_need_regenerate(true);
                }
                ui.end_row();

                ui.label(t!("ui.label.parameters.isolation.overlap"));
                if ui
                    .add(
                        egui::DragValue::new(&mut self.parameters.isolation_overlap)
                            .range(0..=100)
                            .speed(1)
                            .suffix(" %"),
                    )
                    .on_hover_text(t!("ui.tooltip.parameters.isolation.overlap"))
                    .changed()
                {
                    self.status_bar.set_need_regenerate(true);
                }
                ui.end_row();
            });
    }

    fn ui_right_parameters_outline(&mut self, ui: &mut egui::Ui) {
        title_label(
            ui,
            &t!("ui.label.parameters.outline.title").to_string(),
            true,
        );
        ui.separator();

        egui::Grid::new("outline_parameter_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label(t!("ui.label.parameters.outline.depth"));
                if ui
                    .add(
                        egui::DragValue::new(&mut self.parameters.outline_depth)
                            .range(0.1..=20.0)
                            .speed(0.1)
                            .suffix(" mm"),
                    )
                    .on_hover_text(t!("ui.tooltip.parameters.outline.depth"))
                    .changed()
                {
                    self.status_bar.set_need_regenerate(true);
                }
                ui.end_row();
            });
    }

    fn ui_right_parameters_drill(&mut self, ui: &mut egui::Ui) {
        title_label(ui, &t!("ui.label.parameters.drill.title").to_string(), true);
        ui.separator();

        egui::Grid::new("drill_parameter_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label(t!("ui.label.parameters.drill.drill_peck"));
                ui.add(
                    egui::DragValue::new(&mut self.parameters.drill_peck_step)
                        .range(0.1..=10.0)
                        .speed(0.1)
                        .suffix(" mm"),
                )
                .on_hover_text(t!("ui.tooltip.parameters.drill_peck"));
                ui.end_row();

                ui.label(t!("ui.label.parameters.drill.drill_depth"));
                ui.add(
                    egui::DragValue::new(&mut self.parameters.drill_depth)
                        .range(0.1..=20.0)
                        .speed(0.1)
                        .suffix(" mm"),
                )
                .on_hover_text(t!("ui.tooltip.parameters.drill_depth"));
                ui.end_row();
            });
    }

    fn ui_right_export(&mut self, ui: &mut egui::Ui) {
        title_label(ui, &t!("ui.label.exports").to_string(), true);
        ui.separator();

        ui.horizontal(|ui| {
            let output_dir_name = self
                .files
                .output_folder
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| Some(n.to_string_lossy()))
                .unwrap_or(t!("ui.label.select_output_folder"));

            let output_dir = self
                .files
                .output_folder
                .as_ref()
                .and_then(|n| Some(n.to_string_lossy()))
                .unwrap_or(t!("ui.label.select_output_folder"));

            ui.colored_label(
                if self.files.output_folder.is_some() {
                    SUCCESS
                } else {
                    WARNING
                },
                "●",
            );

            if ui.link(output_dir_name).on_hover_text(output_dir).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.files.output_folder = Some(path);
                }
            }
        });

        ui.add_space(8.0);

        Self::export_row(
            ui,
            &t!("ui.label.export_isolation_toolpath").to_string(),
            &mut self.parameters.export_isolation,
        );
        Self::export_row(
            ui,
            &t!("ui.label.export_outline_toolpath").to_string(),
            &mut self.parameters.export_outline,
        );
        Self::export_row(
            ui,
            &t!("ui.label.export_drill").to_string(),
            &mut self.parameters.export_drill,
        );

        ui.add_space(16.0);
        ui.add_enabled_ui(
            !self.status_bar.is_need_regenerate() && self.files.output_folder.is_some(),
            |ui| {
                if button_primary(ui, &t!("ui.button.export_files").to_string(), true).clicked() {
                    self.export_to_gcode();
                }
            },
        );
    }

    fn export_row(ui: &mut egui::Ui, label: &str, active: &mut bool) {
        ui.horizontal(|ui| {
            if *active {
                ui.colored_label(SUCCESS, "■");
            } else {
                ui.colored_label(TEXT_SECONDARY, "■");
            }

            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                toggle(ui, active);
            });
        });
    }

    fn ui_canvas(&mut self, ui: &mut egui::Ui) {
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

        let rect = response.rect;

        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(13, 26, 13));

        let transform = PcbTransform::new(
            // &self.layers.toolpath_generator,
            rect,
            self.canvas.zoom,
            self.canvas.offset,
        );

        draw_axes(&painter, &transform);

        // Draw layers
        if self.layers.show_copper {
            if let Some(generator) = &self.layers.isolation_generator {
                draw_paths(&painter, &generator.traces, &transform, COPPER, 1.0);
            }
        }

        if self.layers.show_copper_toolpaths {
            if let Some(v) = &self.layers.isolation {
                for p in v {
                    draw_paths_stroke(&painter, p, &transform, SUCCESS, 1.0);
                }
            }
        }

        if self.layers.show_outline {
            if let Some(generator) = &self.layers.outline_generator {
                draw_paths_stroke(&painter, &generator.traces, &transform, ACCENT_TEXT, 1.0);
            }
        }

        if self.layers.show_outline_toolpaths {
            if let Some(v) = &self.layers.outline {
                for p in v {
                    draw_paths_stroke(&painter, p, &transform, WARNING, 1.0);
                }
            }
        }

        if self.layers.show_drill {
            if let Some(drill_layer) = &self.layers.drill {
                for hole in &drill_layer.holes {
                    painter.circle_filled(
                        transform.to_screen(hole.x, hole.y),
                        (hole.diameter / 2.0) as f32 * self.canvas.zoom,
                        TEXT_PRIMARY,
                    );
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
                    if button(ui, "−", false).clicked() {
                        self.zoom_on_position(0.95, Vec2::ZERO);
                    }
                    if button(ui, "+", false).clicked() {
                        self.zoom_on_position(1.05, Vec2::ZERO);
                    }
                    if button(ui, "⊡", false).clicked() {
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

    fn isolation_tool_name(&self) -> String {
        match self.tools.isolation_tool {
            SelectedTool::VBit(_) | SelectedTool::EndMill(_) => {
                self.tool_name(&self.tools.isolation_tool)
            }
            SelectedTool::Drill(_) | SelectedTool::None => self.tool_name(&SelectedTool::None),
        }
    }

    fn outline_tool_name(&self) -> String {
        match self.tools.outline_tool {
            SelectedTool::EndMill(_) => self.tool_name(&self.tools.outline_tool),
            SelectedTool::VBit(_) | SelectedTool::Drill(_) | SelectedTool::None => {
                self.tool_name(&SelectedTool::None)
            }
        }
    }

    fn drill_tool_name(&self) -> String {
        match self.tools.drill_tool {
            SelectedTool::Drill(_) => self.tool_name(&self.tools.drill_tool),
            SelectedTool::VBit(_) | SelectedTool::EndMill(_) | SelectedTool::None => {
                self.tool_name(&SelectedTool::None)
            }
        }
    }

    fn tool_name(&self, tool: &SelectedTool) -> String {
        match tool {
            SelectedTool::VBit(id) => self
                .tools
                .tool_library
                .vbits
                .get(*id)
                .map_or(t!("ui.tool.pick").into(), |v| v.to_string()),

            SelectedTool::EndMill(id) => self
                .tools
                .tool_library
                .end_mills
                .get(*id)
                .map_or(t!("ui.tool.pick").into(), |v| v.to_string()),

            SelectedTool::Drill(id) => self
                .tools
                .tool_library
                .drill_bits
                .get(*id)
                .map_or(t!("ui.tool.pick").into(), |v| v.to_string()),

            SelectedTool::None => t!("ui.tool.pick").into(),
        }
    }

    fn run_toolpath_generation(&mut self) {
        if self.files.gerber_copper_path.is_some()
            && self.tools.isolation_tool != SelectedTool::None
        {
            let tool_vbit = match self.tools.isolation_tool {
                SelectedTool::VBit(id) => self.tools.tool_library.vbits.get(id).cloned(),
                _ => None,
            };

            let tool_endmill = match self.tools.isolation_tool {
                SelectedTool::EndMill(id) => self.tools.tool_library.end_mills.get(id).cloned(),
                _ => None,
            };

            if let Some(generator) = self.layers.isolation_generator.as_ref() {
                if let Some(tool) = tool_vbit {
                    self.layers.isolation = Some(generator.generate_toolpaths(
                        &tool,
                        self.parameters.isolation_depth,
                        self.parameters.isolation_passes,
                        self.parameters.isolation_overlap / 100.0,
                    ));
                } else if let Some(tool) = tool_endmill {
                    self.layers.isolation = Some(generator.generate_toolpaths(
                        &tool,
                        self.parameters.isolation_depth,
                        self.parameters.isolation_passes,
                        self.parameters.isolation_overlap / 100.0,
                    ));
                } else {
                    log::error!("{}", t!("ui.error.bad_isolation_tool"));
                    self.layers.isolation = None;
                    return;
                }
            }
        } else {
            log::warn!("{}", t!("ui.warn.no_isolation_toolpath"));
        }

        if self.files.gerber_outline_path.is_some() && self.tools.outline_tool != SelectedTool::None
        {
            let tool_endmill = match self.tools.outline_tool {
                SelectedTool::EndMill(id) => self.tools.tool_library.end_mills.get(id).cloned(),
                _ => None,
            };

            if let Some(generator) = self.layers.outline_generator.as_ref() {
                if let Some(tool) = tool_endmill {
                    self.layers.outline = Some(generator.generate_toolpaths(
                        &tool,
                        self.parameters.isolation_depth,
                        1,
                        self.parameters.isolation_overlap / 100.0,
                    ));
                } else {
                    log::error!("{}", t!("ui.error.bad_outline_tool"));
                    self.layers.outline = None;
                    return;
                }
            }
        } else {
            log::warn!("{}", t!("ui.warn.no_outline_toolpath"));
        }

        self.status_bar.set_need_regenerate(false);
    }

    fn export_to_gcode(&self) {
        if self.files.output_folder.is_none() {
            log::warn!("{}", t!("ui.warn.no_output_folder"));
            return;
        }

        if self.parameters.export_isolation
            && self.layers.isolation.is_some()
            && self.tools.isolation_tool != SelectedTool::None
        {
            let gcode;
            if let SelectedTool::VBit(id) = self.tools.isolation_tool {
                gcode = generate_isolation_gcode(
                    self.layers.isolation.as_ref().unwrap(),
                    &self.tools.tool_library.vbits[id],
                    self.parameters.isolation_depth,
                    self.parameters.z_safe,
                    self.parameters.z_finish,
                );
            } else if let SelectedTool::EndMill(id) = self.tools.isolation_tool {
                gcode = generate_isolation_gcode(
                    self.layers.isolation.as_ref().unwrap(),
                    &self.tools.tool_library.end_mills[id],
                    self.parameters.isolation_depth,
                    self.parameters.z_safe,
                    self.parameters.z_safe,
                );
            } else {
                log::error!("{}", t!("ui.error.bad_isolation_tool"));
                return;
            }

            let gcode_file = self
                .files
                .output_folder
                .as_ref()
                .unwrap()
                .join("isolation.nc");

            match std::fs::write(&gcode_file, gcode) {
                Ok(_) => log::info!(
                    "{}",
                    t!(
                        "ui.info.isolation_write",
                        path = gcode_file.to_string_lossy()
                    )
                ),
                Err(e) => log::error!("{}", t!("ui.error.fail_write_isolation", e = e.to_string())),
            }
        }

        if self.parameters.export_outline
            && self.layers.outline.is_some()
            && self.tools.outline_tool != SelectedTool::None
        {
            let gcode;
            if let SelectedTool::EndMill(id) = self.tools.outline_tool {
                gcode = generate_isolation_gcode(
                    self.layers.outline.as_ref().unwrap(),
                    &self.tools.tool_library.end_mills[id],
                    self.parameters.outline_depth,
                    self.parameters.z_safe,
                    self.parameters.z_safe,
                );
            } else {
                log::error!("{}", t!("ui.error.bad_outline_tool"));
                return;
            }

            let gcode_file = self
                .files
                .output_folder
                .as_ref()
                .unwrap()
                .join("outline.nc");
            match std::fs::write(&gcode_file, gcode) {
                Ok(_) => log::info!(
                    "{}",
                    t!("ui.info.outline_write", path = gcode_file.to_string_lossy())
                ),
                Err(e) => log::error!("{}", t!("ui.error.fail_write_outline", e = e.to_string())),
            }
        }

        if self.parameters.export_drill
            && self.layers.drill.is_some()
            && self.tools.drill_tool != SelectedTool::None
        {
            let gcode;
            if let SelectedTool::Drill(id) = self.tools.drill_tool {
                gcode = generate_drill_gcode(
                    self.layers.drill.as_ref().unwrap(),
                    &self.tools.tool_library.drill_bits[id],
                    self.parameters.drill_depth,
                    self.parameters.z_safe,
                    self.parameters.z_safe,
                    self.parameters.drill_peck_step,
                );
            } else {
                log::error!("{}", t!("ui.error.bad_drill_tool"));
                return;
            }

            let gcode_file = self.files.output_folder.as_ref().unwrap().join("drill.nc");
            match std::fs::write(&gcode_file, gcode) {
                Ok(_) => log::info!(
                    "{}",
                    t!("ui.info.drill_write", path = gcode_file.to_string_lossy())
                ),
                Err(e) => log::error!("{}", t!("ui.error.fail_write_drill", e = e.to_string())),
            }
        }
    }

    fn center_view_to_board(&mut self) {
        let paths: &Paths;

        if self.layers.outline.is_some() && self.layers.outline.as_ref().unwrap().len() > 0 {
            let outline = self.layers.outline.as_ref().unwrap();
            let mut biggest_paths = outline.first().unwrap();

            for p in outline {
                if biggest_paths.signed_area().abs() < p.signed_area().abs() {
                    biggest_paths = p;
                }
            }

            paths = biggest_paths;
        } else if self.layers.isolation.is_some()
            && self.layers.isolation.as_ref().unwrap().len() > 0
        {
            let isolation = self.layers.isolation.as_ref().unwrap();
            let mut biggest_paths = isolation.first().unwrap();

            for p in isolation {
                if biggest_paths.signed_area().abs() < p.signed_area().abs() {
                    biggest_paths = p;
                }
            }

            paths = biggest_paths;
        } else if let Some(generator) = &self.layers.outline_generator {
            paths = &generator.traces;
        } else if let Some(generator) = &self.layers.isolation_generator {
            paths = &generator.traces;
        } else {
            return;
        }

        let (min_x, min_y, max_x, max_y) = bounding_box(paths);
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

impl eframe::App for CopperCrabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let screen_rect = ui.ctx().viewport_rect();
        self.app_config.window_width = screen_rect.width();
        self.app_config.window_height = screen_rect.height();

        self.status_bar.show(ui);
        panel_logs(ui, &self.log_buffer);

        egui::Panel::left("left_panel")
            .resizable(false)
            .min_size(250.0)
            .show_inside(ui, |ui| {
                self.ui_left(ui);
            });

        egui::Panel::right("right_panel")
            .resizable(false)
            .min_size(200.0)
            .show_inside(ui, |ui| {
                self.ui_right(ui);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.ui_canvas(ui);
        });

        self.tool_library_ui.show(
            ui.ctx(),
            &mut self.tools.tool_library_open,
            &mut self.tools.tool_library,
        );
    }

    fn on_exit(&mut self) {
        self.app_config.context_layer = self.layers.clone();
        self.app_config.context_parameter = self.parameters.clone();
        self.app_config.save();
    }
}

fn bounding_box(paths: &Paths) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for path in paths.iter() {
        for point in path.iter() {
            min_x = min_x.min(point.x());
            min_y = min_y.min(point.y());
            max_x = max_x.max(point.x());
            max_y = max_y.max(point.y());
        }
    }

    (min_x, min_y, max_x, max_y)
}
