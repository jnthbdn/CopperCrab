use rust_i18n::t;

use egui::{Id, Layout};

use crate::ui::{AVAILABLE_LOCALE, app_config::AppTheme};

#[derive(Debug)]
pub struct StatusBar {
    mouse_x: f32,
    mouse_y: f32,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            mouse_x: Default::default(),
            mouse_y: Default::default(),
        }
    }
}

impl StatusBar {
    pub fn set_mouse_pcb(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }

    pub fn show(&self, ui: &mut egui::Ui, app_locale: &mut String, app_theme: &mut AppTheme) {
        egui::Panel::bottom(Id::new("info_bar")).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{}: {:.3} mm, {:.3} mm",
                    t!("ui.status_bar.mouse"),
                    self.mouse_x,
                    self.mouse_y
                ));

                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    let before = app_locale.clone();

                    egui::ComboBox::from_id_salt("locale_settings")
                        .selected_text(get_locale_name(&app_locale))
                        .wrap()
                        .show_ui(ui, |ui| {
                            for (locale, name) in AVAILABLE_LOCALE {
                                ui.selectable_value(app_locale, locale.to_string(), name);
                            }
                        });

                    if *app_locale != before {
                        rust_i18n::set_locale(&app_locale);
                    }

                    ui.add_sized(
                        [160.0, ui.spacing().interact_size.y],
                        elegance::Select::new("unit", app_theme).options([
                            (AppTheme::Slate, "Slate (dark)"),
                            (AppTheme::Charcoal, "Charcoal (dark)"),
                            (AppTheme::Frost, "Frost (light)"),
                            (AppTheme::Paper, "Paper (light)"),
                        ]),
                    );
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
