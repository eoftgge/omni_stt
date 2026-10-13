use crate::gui::theme;
use eframe::egui::{self, Grid, Response, RichText, Ui};

const LABEL_WIDTH: f32 = 110.0;

pub(super) fn section(ui: &mut Ui, title: Option<&str>, add_contents: impl FnOnce(&mut Ui)) {
    let Some(title) = title else {
        add_contents(ui);
        return;
    };

    let widgets = &ui.visuals().widgets.noninteractive;
    let (stroke, radius) = (widgets.bg_stroke, widgets.corner_radius);

    egui::Frame::new()
        .stroke(stroke)
        .corner_radius(radius)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(15.0).strong().color(theme::TEXT));
            add_contents(ui);
        });
}

pub(super) fn row(ui: &mut Ui, text: &str, widget: impl egui::Widget) -> Response {
    let label_response = label(ui, text);
    let widget_response = ui.add(widget);
    ui.end_row();
    label_response | widget_response
}

pub(super) fn settings_grid(id: &str) -> Grid {
    Grid::new(id)
        .num_columns(2)
        .spacing([12.0, 10.0])
        .min_col_width(LABEL_WIDTH)
}

pub(super) fn label(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Label::new(text).extend())
}
