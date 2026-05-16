use egui::Id;

use crate::ui::colors::*;

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

    pub fn show(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom(Id::new("info_bar")).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Mouse: {:.3} mm, {:.3} mm",
                    self.mouse_x, self.mouse_y
                ));
                ui.label(" | ");

                if self.need_regen {
                    ui.colored_label(WARNING, "Toolpaths outdated");
                } else {
                    ui.colored_label(SUCCESS, "Toolpath up to date");
                }
            })
        });
    }
}
