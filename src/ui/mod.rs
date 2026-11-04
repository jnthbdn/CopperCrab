use eframe::epaint::CornerRadius;
use egui_ltreeview::TreeViewState;
use elegance::Theme;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    core::tools::library::ToolLibrary,
    logger::LogEntry,
    ui::{
        app_config::AppConfig,
        log_ui::panel_logs,
        panels::{CanvasState, FileOperationId},
        project::SourceFile,
        widget::{status_bar::StatusBar, tool_library::ToolLibraryUi},
    },
};

pub mod app_config;
mod colors;
mod log_ui;
mod panels;
mod pcb_transform;
mod project;
mod toolpaths;
mod widget;

pub(crate) use project::CommonParameters;

const TOOLS_FILENAME: &str = "tools.toml";
const AVAILABLE_LOCALE: [(&str, &str); 3] =
    [("en", "English"), ("fr", "Français"), ("ru", "Russian")];

#[derive(Debug, Default, Serialize, Deserialize)]
struct ContextTools {
    tool_library_open: bool,
    tool_library: ToolLibrary,
}

#[derive(Debug)]
pub struct CopperCrabApp {
    files: Vec<SourceFile>,
    tree_files_states: TreeViewState<FileOperationId>,
    tools: ContextTools,
    canvas: CanvasState,
    tool_library_ui: ToolLibraryUi,
    status_bar: StatusBar,
    log_buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    app_config: AppConfig,
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

        apply_theme(&cc.egui_ctx, &app_config.app_theme);

        Self {
            files: Default::default(),
            tree_files_states: Default::default(),
            tools: ContextTools {
                tool_library: ToolLibrary::new(&config_folder),
                ..Default::default()
            },
            canvas: Default::default(),
            tool_library_ui: Default::default(),
            status_bar: Default::default(),
            log_buffer,
            app_config,
        }
    }

    fn ui_top_menu(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_menu_panel")
            .resizable(false)
            .min_size(28.0)
            .max_size(28.0)
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    let file_label = t!("ui.menu.file").to_string();
                    let view_label = t!("ui.menu.view").to_string();
                    let new_project_label = t!("ui.menu.new_project").to_string();
                    let open_project_label = t!("ui.menu.open_project").to_string();
                    let exit_label = t!("ui.menu.exit").to_string();
                    let reset_zoom_label = t!("ui.menu.reset_zoom").to_string();
                    let center_board_label = t!("ui.menu.center_board").to_string();

                    ui.menu_button(file_label, |ui| {
                        if ui.button(new_project_label).clicked() {
                            ui.close();
                        }
                        if ui.button(open_project_label).clicked() {
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(exit_label).clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button(view_label, |ui| {
                        if ui.button(reset_zoom_label).clicked() {
                            self.canvas.zoom = 1.0;
                            self.canvas.offset = egui::Vec2::ZERO;
                            ui.close();
                        }
                        if ui.button(center_board_label).clicked() {
                            self.center_view_to_board();
                            ui.close();
                        }
                    });
                });
            });
    }
}

impl eframe::App for CopperCrabApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        apply_theme(ui.ctx(), &self.app_config.app_theme);

        let screen_rect = ui.ctx().viewport_rect();
        self.app_config.window_width = screen_rect.width();
        self.app_config.window_height = screen_rect.height();

        self.ui_top_menu(ui);
        self.status_bar.show(
            ui,
            &mut self.app_config.app_locale,
            &mut self.app_config.app_theme,
        );
        panel_logs(ui, &self.log_buffer);

        egui::Panel::left("left_panel")
            .resizable(true)
            .default_size(300.0)
            .min_size(280.0)
            .max_size(400.0)
            .show(ui, |ui| {
                self.ui_left(ui);
            });

        egui::Panel::right("right_panel")
            .resizable(true)
            .default_size(280.0)
            .min_size(220.0)
            .max_size(600.0)
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
fn apply_theme(ctx: &egui::Context, app_theme: &app_config::AppTheme) {
    let mut theme = match app_theme {
        app_config::AppTheme::Slate => Theme::slate(),
        app_config::AppTheme::Charcoal => Theme::charcoal(),
        app_config::AppTheme::Frost => Theme::frost(),
        app_config::AppTheme::Paper => Theme::paper(),
    };

    let btn_bg = egui::Color32::from_rgb(38, 38, 38);
    let btn_hover = egui::Color32::from_rgb(51, 51, 51);

    theme.palette.blue = btn_bg;
    theme.palette.blue_hover = btn_hover;
    theme.palette.green = btn_bg;
    theme.palette.green_hover = btn_hover;
    theme.palette.red = btn_bg;
    theme.palette.red_hover = btn_hover;
    theme.palette.purple = btn_bg;
    theme.palette.purple_hover = btn_hover;
    theme.palette.amber = btn_bg;
    theme.palette.amber_hover = btn_hover;
    theme.card_radius = 20.0;

    theme.install(ctx);

    ctx.global_style_mut(|style| {
        for visuals in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            visuals.bg_fill = btn_bg;
            visuals.weak_bg_fill = btn_bg;
            visuals.bg_stroke = egui::Stroke::NONE;
            visuals.corner_radius = 8u8.into();
        }
    });
}
