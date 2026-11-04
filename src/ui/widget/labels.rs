use egui::{DragValue, Response, Ui, WidgetText};
use elegance::Theme;

pub fn title_label(ui: &mut egui::Ui, label: &str, add_space: bool) -> egui::Response {
    if add_space {
        ui.add_space(16.0);
    }
    let theme = elegance::Theme::current(ui.ctx());


    ui.label(theme.heading_text(label))
}

pub fn dragvalue_with_label(
    ui: &mut Ui,
    drag_value: DragValue,
    label: impl Into<String>,
    tooltip: impl Into<WidgetText>,
) -> Response {
    let theme = Theme::current(ui.ctx());
    let mut resp = None;

    ui.vertical(|ui| {
        ui.label(theme.muted_text(label));
        resp = Some(ui.add(drag_value).on_hover_text(tooltip));
    });

    resp.unwrap()
}
