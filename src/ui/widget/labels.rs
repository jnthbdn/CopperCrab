pub fn title_label(ui: &mut egui::Ui, label: &str, add_space: bool) -> egui::Response {
    if add_space {
        ui.add_space(16.0);
    }
    let theme = elegance::Theme::current(ui.ctx());

    // ui.label(RichText::new(label).color(COPPER_LIGHT).size(16.0))
    ui.label(theme.heading_text(label))
}
