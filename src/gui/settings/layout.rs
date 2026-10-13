use crate::gui::theme;
use eframe::egui::{self, CornerRadius, Grid, Response, RichText, Ui};

/// egui paints a checkbox with the shared widget corner radius, and at the
/// 14 px icon size the theme's 6 px is all but a circle. Square it off around
/// this widget alone, so buttons and fields keep the rounding they were given.
pub(super) struct Squared<W>(pub W);

impl<W: egui::Widget> egui::Widget for Squared<W> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope(|ui| {
            let radius = CornerRadius::same(2);
            let w = &mut ui.visuals_mut().widgets;
            w.noninteractive.corner_radius = radius;
            w.inactive.corner_radius = radius;
            w.hovered.corner_radius = radius;
            w.active.corner_radius = radius;

            ui.add(self.0)
        })
        .inner
    }
}

pub(super) fn section(ui: &mut Ui, title: Option<&str>, add_contents: impl FnOnce(&mut Ui)) {
    if let Some(title) = title {
        ui.label(RichText::new(title).size(13.0).strong().color(theme::LABEL));
        ui.add_space(2.0);
    }
    add_contents(ui);
    ui.add_space(10.0);
}

pub(super) fn row(ui: &mut Ui, text: &str, widget: impl egui::Widget) -> Response {
    let label_response = label(ui, text);
    let widget_response = ui.add(widget);
    ui.end_row();
    label_response | widget_response
}

pub(super) fn settings_grid(id: &str) -> Grid {
    Grid::new(id).num_columns(2).spacing([12.0, 10.0])
}

pub(super) fn label(ui: &mut Ui, text: &str) -> Response {
    ui.add(egui::Label::new(text).extend())
}
