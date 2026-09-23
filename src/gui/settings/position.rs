use super::layout::{label, section, settings_grid};
use crate::settings::anchor::Anchor;
use crate::settings::ui::SettingsUI;
use crate::gui::monitor::Monitor;
use eframe::egui::{self, Button, ComboBox, DragValue, Grid, Response, RichText, Ui, vec2};

const SAME_AS_SETTINGS: &str = "Same as settings window";

pub(super) fn ui_section_position(
    ui: &mut Ui,
    settings_ui: &mut SettingsUI,
    monitors: &mut Vec<Monitor>,
) {
    section(ui, "Position", false, |ui| {
        settings_grid("position_grid").show(ui, |ui| {
            label(ui, "Monitor:");
            let selected = match &settings_ui.monitor {
                None => SAME_AS_SETTINGS.to_owned(),
                Some(name) => monitors
                    .iter()
                    .find(|monitor| &monitor.name == name)
                    .map_or_else(|| "Not connected".to_owned(), |m| m.label.clone()),
            };
            let combo = ComboBox::from_id_salt("overlay_monitor")
                .selected_text(selected)
                .width(220.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut settings_ui.monitor, None, SAME_AS_SETTINGS);
                    for monitor in monitors.iter() {
                        ui.selectable_value(
                            &mut settings_ui.monitor,
                            Some(monitor.name.clone()),
                            &monitor.label,
                        );
                    }
                });
            
            // Re-list on every open: the projector is usually plugged in after
            // the app has started.
            if combo.response.clicked() {
                *monitors = Monitor::list();
            }
            combo.response.on_hover_text(
                "Screen the overlay opens on. If it is unplugged, the overlay \
                 opens where the settings window is.",
            );
            ui.end_row();

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
    let button = Button::new(RichText::new(anchor.glyph()).size(16.0)).min_size(vec2(30.0, 30.0));

    if selected {
        ui.add(button.fill(ui.ctx().global_style().visuals.selection.bg_fill))
    } else {
        ui.add(button)
    }
}
