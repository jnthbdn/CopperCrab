use rust_i18n::t;

use egui::{Id, Layout};

use crate::ui::{AVAILABLE_LOCALE, colors::*};

#[derive(Debug)]
pub struct StatusBar {
    mouse_x: f32,
    mouse_y: f32,

    need_regen: bool,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            mouse_x: Default::default(),
            mouse_y: Default::default(),
            need_regen: true,
        }
    }
}

impl StatusBar {
    pub fn set_mouse_pcb(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn set_need_regenerate(&mut self, need_regenerate: bool) {
        self.need_regen = need_regenerate;
    }

    pub fn is_need_regenerate(&self) -> bool {
        self.need_regen
    }

    pub fn show(&self, ui: &mut egui::Ui, app_locale: &mut String) {
        egui::Panel::bottom(Id::new("info_bar")).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{}: {:.3} mm, {:.3} mm",
                    t!("ui.status_bar.mouse"),
                    self.mouse_x,
                    self.mouse_y
                ));
                ui.label(" | ");

                if self.need_regen {
                    ui.colored_label(WARNING, t!("ui.status_bar.toolpaths_outdated"));
                } else {
                    ui.colored_label(SUCCESS, t!("ui.status_bar.toolpath_up_to_date"));
                }

                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    let before = app_locale.clone();
                    egui::ComboBox::from_id_salt("locale_settings")
                        .selected_text(get_locale_name(&app_locale))
                        .show_ui(ui, |ui| {
                            for (locale, name) in AVAILABLE_LOCALE {
                                ui.selectable_value(app_locale, locale.to_string(), name);
                            }
                        });

                    if *app_locale != before {
                        rust_i18n::set_locale(&app_locale);
                    }
                });
            })
        });
    }
}

fn get_locale_name(locale: &str) -> &str {
    for (l, name) in AVAILABLE_LOCALE {
        if l == locale {
            return name;
        }
    }

    "---"
}
