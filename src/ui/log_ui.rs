use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use elegance::{Button, Theme};
use rust_i18n::t;

use crate::{logger::LogEntry, ui::widget::labels::title_label};

pub fn panel_logs(ui: &mut egui::Ui, log_buffer: &Arc<Mutex<VecDeque<LogEntry>>>) {
    egui::Panel::bottom("log_panel")
        .resizable(true)
        .min_size(80.0)
        .default_size(120.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
            
                title_label(ui, &t!("ui.label.logs").to_string(), false);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(Button::new(t!("ui.button.clear_logs"))).clicked() {
                        if let Ok(mut logs) = log_buffer.lock() {
                            logs.clear();
                        }
                    }
                });
            });

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    ui.set_min_width(ui.available_width());

                    let palette = Theme::current(ui.ctx()).palette;

                    if let Ok(logs) = log_buffer.lock() {
                        for (i, entry) in logs.iter().enumerate() {
                            let color = match entry.level {
                                log::Level::Error => palette.danger,
                                log::Level::Warn => palette.warning,
                                log::Level::Info => palette.text_muted,
                                _ => palette.text_faint,
                            };

                            ui.push_id(i, |ui| {
                                ui.colored_label(color, &entry.message);
                            });
                        }
                    }
                });
        });
}
