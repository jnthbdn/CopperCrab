use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use egui_ltreeview::TreeViewState;
use elegance::Theme;
use serde::{Deserialize, Serialize};

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
const AVAILABLE_LOCALE: [(&str, &str); 2] = [("en", "English"), ("fr", "Français")];

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
            canvas: Default::default(),
            tool_library_ui: Default::default(),
            status_bar: Default::default(),
            log_buffer: log_buffer,
            app_config,
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
