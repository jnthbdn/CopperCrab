use elegance::{Accent, Button};
use rust_i18n::t;

use crate::ui::{CopperCrabApp, project::ExportFormat};

impl CopperCrabApp {
    pub(crate) fn ui_export_settings(&mut self, ctx: &egui::Context) {
        if !self.export_settings_open {
            return;
        }

        let width = ctx.viewport_rect().width() * 0.7;

        let modal = egui::Modal::new(egui::Id::new("modal_export_settings"))
            .frame(
                egui::Frame::popup(&ctx.global_style())
                    .inner_margin(egui::Margin::symmetric(16, 16)),
            )
            .show(ctx, |ui| {
                ui.set_width(width);

                ui.horizontal(|ui| {
                    let _ = ui.selectable_label(true, t!("ui.label.export_settings.title"));
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let format = self.export_settings.format;
                    let (header_label, footer_label) = format.ui_labels();

                    egui::Grid::new("export_settings_grid")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label(t!("ui.label.export_settings.format"));
                            egui::ComboBox::from_id_salt("export_format")
                                .selected_text(format.label())
                                .width(ui.available_width())
                                .show_ui(ui, |ui| {
                                    for f in ExportFormat::ALL {
                                        ui.selectable_value(
                                            &mut self.export_settings.format,
                                            *f,
                                            f.label(),
                                        );
                                    }
                                });
                            ui.end_row();
                        });

                    ui.add_space(8.0);

                    let payload = self.export_settings.payload_mut(format);

                    ui.label(header_label);
                    ui.add(
                        egui::TextEdit::multiline(&mut payload.header)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                            .code_editor(),
                    );

                    ui.add_space(8.0);

                    ui.label(footer_label);
                    ui.add(
                        egui::TextEdit::multiline(&mut payload.footer)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                            .code_editor(),
                    );
                });

                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui
                        .add(Button::new(t!("ui.button.close")).accent(Accent::Blue))
                        .clicked()
                    {
                        self.export_settings_open = false;
                    }
                    if ui
                        .add(Button::new(t!("ui.button.save")).accent(Accent::Blue))
                        .clicked()
                    {
                        self.save_project_interactive();
                    }
                });
            });

        if modal.should_close() {
            self.export_settings_open = false;
        }
    }
}
