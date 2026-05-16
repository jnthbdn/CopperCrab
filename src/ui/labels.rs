use egui::RichText;

use crate::ui::colors::COPPER_LIGHT;

pub fn title_label(ui: &mut egui::Ui, label: &str, add_space: bool) -> egui::Response {
    if add_space {
        ui.add_space(16.0);
    }

    ui.label(RichText::new(label).color(COPPER_LIGHT).size(16.0))
}
