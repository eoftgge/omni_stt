use crate::gui::theme;
use eframe::egui::{self, CollapsingHeader, Grid, Response, RichText, Ui};

pub(super) fn section(ui: &mut Ui, title: &str, default_open: bool, add_contents: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(RichText::new(title).size(15.0).strong().color(theme::TEXT))
        .default_open(default_open)
        .show(ui, add_contents);
    ui.add_space(6.0);
}

pub(super) fn row(ui: &mut Ui, label: &str, widget: impl egui::Widget) -> Response {
    let label_response = ui.add(egui::Label::new(label).extend());
    let widget_response = ui.add(widget);
    ui.end_row();
    label_response | widget_response
}

pub(super) fn settings_grid(id: &str) -> Grid {
    Grid::new(id).num_columns(2).spacing([12.0, 10.0])
}
