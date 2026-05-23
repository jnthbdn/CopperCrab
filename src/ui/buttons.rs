use std::path::PathBuf;

use egui::{Color32, Layout, RichText};

use crate::{
    core::{
        DrillLayer,
        parser::{excellon::load_excellon, gerber::load_gerber},
        toolpath::ToolpathGenerator,
    },
    ui::colors::{
        ACCENT, ACCENT_BORDER, ACCENT_TEXT, BG_TERTIARY, BORDER_DEFAULT, ERROR, SUCCESS,
        TEXT_PRIMARY, WARNING,
    },
};

pub fn button(ui: &mut egui::Ui, label: &str, full_width: bool) -> egui::Response {
    raw_styled_button(
        ui,
        label,
        full_width,
        TEXT_PRIMARY,
        BG_TERTIARY,
        BORDER_DEFAULT, // egui::Color32::from_rgb(70, 70, 70),
                        // egui::Color32::from_rgb(46, 46, 46),
    )
}

pub fn button_primary(ui: &mut egui::Ui, label: &str, full_width: bool) -> egui::Response {
    raw_styled_button(
        ui,
        label,
        full_width,
        ACCENT_TEXT,
        ACCENT,
        ACCENT_BORDER, // egui::Color32::from_rgb(42, 122, 191),
                       // egui::Color32::from_rgb(26, 90, 138),
    )
}

pub fn button_danger(ui: &mut egui::Ui, label: &str, full_width: bool) -> egui::Response {
    raw_styled_button(
        ui,
        label,
        full_width,
        ERROR,
        BG_TERTIARY,
        ERROR, // egui::Color32::from_rgb(42, 122, 191),
               // egui::Color32::from_rgb(26, 90, 138),
    )
}

fn raw_styled_button(
    ui: &mut egui::Ui,
    label: &str,
    full_width: bool,
    text_color: Color32,
    bg_color: Color32,
    stroke_color: Color32,
) -> egui::Response {
    let mut response = None;
    ui.scope(|ui| {
        ui.style_mut().spacing.button_padding = egui::vec2(10.0, 4.0);
        if full_width {
            let width = ui.available_width();
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                response = Some(
                    ui.add(
                        egui::Button::new(RichText::new(label).color(text_color))
                            .fill(bg_color)
                            .stroke(egui::Stroke::new(1.0, stroke_color))
                            .corner_radius(3.0)
                            .min_size(egui::vec2(width, 0.0)),
                    ),
                );
            });
        } else {
            response = Some(
                ui.add(
                    egui::Button::new(RichText::new(label).color(text_color))
                        .fill(bg_color)
                        .stroke(egui::Stroke::new(1.0, stroke_color))
                        .corner_radius(3.0),
                ),
            );
        }
    });
    response.unwrap()
}

pub fn toggle(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
    });

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();

        ui.painter().rect(
            rect,
            radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );

        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}

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
                                    log::error!("{}", e);
                                    *gerber_path = None;
                                    *toolpath_generator = None;
                                }
                            };
                        }
                        Err(e) => log::error!("Failed to load gerber: {}", e),
                    }
                    changed = true;
                }
            }

            if toolpath_generator.is_some() {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if button_danger(ui, "🗑", false).clicked() {
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
                        Err(e) => log::error!("Failed to load excellon file: {e}"),
                    }
                }
            }

            if drill_layer.is_some() {
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    if button_danger(ui, "🗑", false).clicked() {
                        *drill_path = None;
                        *drill_layer = None;
                    }
                });
            }
        });
}
