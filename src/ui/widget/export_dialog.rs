use elegance::{Accent, Button};
use rust_i18n::t;

use crate::ui::{
    CopperCrabApp,
    project::{ExportImageFormat, PNG_SCALE_FACTORS},
};

impl CopperCrabApp {
    pub(crate) fn ui_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.export_dialog_open {
            return;
        }

        let mut open = self.export_dialog_open;

        let modal = egui::Modal::new(egui::Id::new("modal_export_dialog"))
            .frame(
                egui::Frame::popup(&ctx.global_style())
                    .inner_margin(egui::Margin::symmetric(16, 16)),
            )
            .show(ctx, |ui| {
                ui.set_min_width(400.0);

                ui.heading(t!("ui.label.export.title"));
                ui.separator();
                ui.add_space(8.0);

                egui::Grid::new("export_dialog_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label(t!("ui.label.export.format"));
                        egui::ComboBox::from_id_salt("export_image_format")
                            .selected_text(self.export_settings.image_format.label())
                            .width(ui.available_width())
                            .show_ui(ui, |ui| {
                                for f in ExportImageFormat::ALL {
                                    ui.selectable_value(
                                        &mut self.export_settings.image_format,
                                        *f,
                                        f.label(),
                                    );
                                }
                            });
                        ui.end_row();

                        if self.export_settings.image_format == ExportImageFormat::Png {
                            ui.label(t!("ui.label.export.png_scale"));
                            let current = self.export_settings.png.scale_factor;
                            egui::ComboBox::from_id_salt("export_png_scale")
                                .selected_text(format!("x{}", current))
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for &factor in PNG_SCALE_FACTORS {
                                        let selected = (current - factor).abs() < f64::EPSILON;
                                        if ui
                                            .selectable_label(selected, format!("x{}", factor))
                                            .clicked()
                                        {
                                            self.export_settings.png.scale_factor = factor;
                                        }
                                    }
                                });
                            ui.end_row();
                        }

                        ui.label("");
                        ui.checkbox(&mut self.export_settings.color.bw, t!("ui.label.export.bw"));
                        ui.end_row();

                        if self.export_settings.color.bw {
                            ui.label("");
                            ui.checkbox(
                                &mut self.export_settings.color.invert,
                                t!("ui.label.export.invert"),
                            );
                            ui.end_row();
                        }

                        ui.label("");
                        ui.checkbox(
                            &mut self.export_settings.color.transparent_bg,
                            t!("ui.label.export.transparent_bg"),
                        );
                        ui.end_row();

                        if !self.export_settings.color.bw
                            && !self.export_settings.color.transparent_bg
                        {
                            ui.label(t!("ui.label.export.background_color"));
                            color_picker(ui, &mut self.export_settings.color.background);
                            ui.end_row();
                        }

                        if !self.export_settings.color.bw {
                            ui.label(t!("ui.label.export.tracks_color"));
                            color_picker(ui, &mut self.export_settings.color.tracks);
                            ui.end_row();
                        }
                    });

                ui.add_space(16.0);
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui
                        .add(Button::new(t!("ui.button.export")).accent(Accent::Blue))
                        .clicked()
                    {
                        self.run_export_interactive();
                        open = false;
                    }
                    if ui.add(Button::new(t!("ui.button.cancel"))).clicked() {
                        open = false;
                    }
                });
            });

        if modal.should_close() {
            open = false;
        }

        self.export_dialog_open = open;
    }

    fn run_export_interactive(&mut self) {
        let format = self.export_settings.image_format;
        let ext = format.extension();

        let file_name = self
            .current_project_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("export")
            .to_string();

        let Some(path) = rfd::FileDialog::new()
            .add_filter(format.label(), &[ext])
            .set_file_name(format!("{file_name}.{ext}"))
            .save_file()
        else {
            return;
        };

        let result = match format {
            ExportImageFormat::Svg => self.export_visible_svg(&path),
            ExportImageFormat::Png => self.export_visible_png(&path),
        };

        match result {
            Ok(()) => log::info!("Exported {}", path.display()),
            Err(e) => log::error!("Export failed: {e}"),
        }
    }
}

fn color_picker(ui: &mut egui::Ui, rgb: &mut [u8; 3]) {
    let mut color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    if ui.color_edit_button_srgba(&mut color).changed() {
        *rgb = [color.r(), color.g(), color.b()];
    }
}
