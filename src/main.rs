use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{
    dirs::get_config_dir,
    logger::MultiLogger,
    ui::{CopperCrabApp, app_config::AppConfig},
};

use eframe;

mod core;
mod dirs;
mod logger;
mod ui;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let log_buffer = Arc::new(Mutex::new(VecDeque::new()));
    let logger = MultiLogger::new(log_buffer.clone());
    let max_level = logger.console.filter();

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(max_level);

    let config_folder = match get_config_dir() {
        Some(f) => f,
        None => {
            log::error!("Failed to fetch the config folder !");
            return;
        }
    };

    log::info!("Config dir: {:?}", config_folder);

    if false == config_folder.exists() {
        if let Err(e) = std::fs::create_dir_all(&config_folder) {
            log::error!("Failed to create config directory ({e})");
            return;
        } else {
            log::info!("Create config directory !");
        }
    }

    let app_config = AppConfig::new(&config_folder);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(format!("CopperCrab - v{VERSION}"))
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../img/icons/png/coppercrab-300.png"
                ))
                .unwrap(),
            )
            .with_inner_size([app_config.window_width, app_config.window_height]),
        // .with_inner_size([1200.0, 700.0]),
        ..eframe::NativeOptions::default()
    };
    let _ = eframe::run_native(
        "CopperCrab",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(CopperCrabApp::new(
                cc,
                log_buffer,
                app_config,
                config_folder,
            )))
        }),
    );
}
