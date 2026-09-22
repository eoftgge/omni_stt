use super::layout::{label, section, settings_grid};
use crate::settings::ui::SettingsUI;
use eframe::egui::{self, vec2, Button, DragValue, Grid, Response, RichText, Ui};
use crate::settings::anchor::Anchor;

pub(super) fn ui_section_position(ui: &mut Ui, settings_ui: &mut SettingsUI) {
    section(ui, "Position", false, |ui| {
        settings_grid("position_grid").show(ui, |ui| {
            ui.add(egui::Label::new("Offset:").extend());
            ui.horizontal(|ui| {
                ui.add(
                    DragValue::new(&mut settings_ui.offset.0)
                        .speed(1.0)
                        .prefix("X: "),
                );
                ui.add(
                    DragValue::new(&mut settings_ui.offset.1)
                        .speed(1.0)
                        .prefix("Y: "),
                );
            });
            ui.end_row();

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                label(ui, "Snap to:");
            });
            ui.vertical(|ui| {
                Grid::new("snap_buttons")
                    .spacing([5.0, 5.0])
                    .show(ui, |ui| {
                        for line in Anchor::ALL.chunks(3) {
                            for &anchor in line {
                                let selected = settings_ui.anchor == anchor;
                                if ui_snap_button(ui, anchor, selected).clicked() {
                                    settings_ui.anchor = anchor;
                                    settings_ui.offset = anchor.default_offset();
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
            ui.end_row();
        });
    });
}

fn ui_snap_button(ui: &mut Ui, anchor: Anchor, selected: bool) -> Response {
    let button =
        Button::new(RichText::new(anchor.glyph()).size(16.0)).min_size(vec2(30.0, 30.0));

    if selected {
        ui.add(button.fill(ui.ctx().global_style().visuals.selection.bg_fill))
    } else {
        ui.add(button)
    }
}
