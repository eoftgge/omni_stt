use crate::gui::settings::layout::{section, settings_grid};
use crate::settings::SettingsUI;
use eframe::egui;
use eframe::egui::{Button, DragValue, Grid, RichText, Ui, vec2};

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
                ui.add(egui::Label::new("Snap to:").extend());
            });
            ui.vertical(|ui| {
                Grid::new("snap_buttons")
                    .spacing([5.0, 5.0])
                    .show(ui, |ui| {
                        let mut btn =
                            |ui: &mut Ui,
                             text: &str,
                             anchor_val: usize,
                             default_offset: (f32, f32)| {
                                let is_selected = settings_ui.anchor == anchor_val;
                                let button = Button::new(RichText::new(text).size(16.0))
                                    .min_size(vec2(30.0, 30.0));

                                let response =
                                    if is_selected {
                                        ui.add(button.fill(
                                            ui.ctx().global_style().visuals.selection.bg_fill,
                                        ))
                                    } else {
                                        ui.add(button)
                                    };
                                if response.clicked() {
                                    settings_ui.anchor = anchor_val;
                                    settings_ui.offset = default_offset;
                                }
                            };

                        let pad = 30.0;
                        btn(ui, "↖", 0, (pad, pad));
                        btn(ui, "↑", 1, (0.0, pad));
                        btn(ui, "↗", 2, (-pad, pad));
                        ui.end_row();

                        btn(ui, "←", 3, (pad, 0.0));
                        btn(ui, "•", 4, (0.0, 0.0));
                        btn(ui, "→", 5, (-pad, 0.0));
                        ui.end_row();

                        btn(ui, "↙", 6, (pad, -pad));
                        btn(ui, "↓", 7, (0.0, -pad));
                        btn(ui, "↘", 8, (-pad, -pad));
                        ui.end_row();
                    });
            });
            ui.end_row();
        });
    });
}
