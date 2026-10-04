use std::path::PathBuf;

use elegance::{Accent, Button};
use rust_i18n::t;

use egui::Layout;

use crate::{
    core::{
        DrillLayer,
        parser::{excellon::load_excellon, gerber::load_gerber},
        toolpath::ToolpathGenerator,
    },
    ui::colors::{SUCCESS, WARNING},
};

pub fn pick_gerber_file(
    ui: &mut egui::Ui,
    gerber_path: &mut Option<PathBuf>,
    toolpath_generator: &mut Option<ToolpathGenerator>,
    extensions: &[&str],
    default_text: &str,
    is_outline: bool,
) -> bool {
    let mut changed = false;

    egui::Grid::new(format!("pickfile_grid_{}", default_text))
        .num_columns(3)
        .min_col_width(0.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            let gerber_label = gerber_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(default_text);

            let gerber_label_path = gerber_path
                .as_ref()
                .and_then(|n| n.to_str())
                .unwrap_or(default_text);

            ui.colored_label(
                if toolpath_generator.is_some() {
                    SUCCESS
                } else {
                    WARNING
                },
                "■",
            );

            if ui
                .link(gerber_label)
                .on_hover_text(gerber_label_path)
                .clicked()
            {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Gerber", extensions)
                    .pick_file()
                {
                    match load_gerber(&path) {
                        Ok(layer) => {
                            *gerber_path = Some(path);

                            match ToolpathGenerator::new(&layer, is_outline) {
                                Ok(generator) => *toolpath_generator = Some(generator),
                                Err(e) => {
                                    log::error!(
                                        "{}",
                                        t!("ui.toolpath.error.fail_generate", e = e.to_string())
                                    );
                                    *gerber_path = None;
                                    *toolpath_generator = None;
                                }
                            };
                        }
                        Err(e) => {
                            log::error!("{}", t!("ui.toolpath.error.fail_load", e = e.to_string()))
                        }
                    }
                    changed = true;
                }
            }

            if toolpath_generator.is_some() {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.add(Button::new("🗑").accent(Accent::Red)).clicked() {
                        changed = true;
                        *gerber_path = None;
                        *toolpath_generator = None;
                    }
                });
            }
        });

    changed
}

pub fn pick_drill_file(
    ui: &mut egui::Ui,
    drill_path: &mut Option<PathBuf>,
    drill_layer: &mut Option<DrillLayer>,
    default_text: &str,
) {
    egui::Grid::new(format!("pickfile_grid_{}", default_text))
        .num_columns(3)
        .min_col_width(0.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            let file_label = drill_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or(default_text);

            let file_label_path = drill_path
                .as_ref()
                .and_then(|n| n.to_str())
                .unwrap_or(default_text);

            ui.colored_label(
                if drill_layer.is_some() {
                    SUCCESS
                } else {
                    WARNING
                },
                "■",
            );

            if ui.link(file_label).on_hover_text(file_label_path).clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Excellon", &["drl", "txt"])
                    .pick_file()
                {
                    match load_excellon(&path) {
                        Ok(layer) => {
                            *drill_path = Some(path);
                            *drill_layer = Some(layer);
                        }
                        Err(e) => {
                            log::error!("{}", t!("ui.excellon.error.fail_load", e = e.to_string()))
                        }
                    }
                }
            }

            if drill_layer.is_some() {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.add(Button::new("🗑").accent(Accent::Red)).clicked() {
                        *drill_path = None;
                        *drill_layer = None;
                    }
                });
            }
        });
}
