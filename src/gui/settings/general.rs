use super::layout::{label, row, section, settings_grid};
use crate::logger::LEVELS;
use crate::settings::general::SettingsGeneral;
use eframe::egui::{Checkbox, ComboBox, Slider, Ui};

pub(super) fn ui_section_general(
    ui: &mut Ui,
    settings_general: &mut SettingsGeneral,
    max_blocks: &mut usize,
) {
    section(ui, Some("Advanced"), |ui| {
        settings_grid("general_grid").show(ui, |ui| {
            row(ui, "Text history:", Slider::new(max_blocks, 1..=10)).on_hover_text(
                "How many finished blocks are kept behind the visible lines. \
                 Raise it if text vanishes too early.",
            );

            label(ui, "Log level:");
            ComboBox::from_id_salt("log_level")
                .selected_text(settings_general.level.to_string())
                .width(80.0)
                .show_ui(ui, |ui| {
                    for level in LEVELS {
                        ui.selectable_value(&mut settings_general.level, *level, level.to_string());
                    }
                });
            ui.end_row();

            row(
                ui,
                "Log to file:",
                Checkbox::without_text(&mut settings_general.log_to_file),
            )
            .on_hover_text("Save logs to a .log file in the app directory");

            row(
                ui,
                "Save transcripts:",
                Checkbox::without_text(&mut settings_general.save_transcripts),
            )
            .on_hover_text("Append finished subtitles to transcripts/omni-YYYY-MM-DD.txt");
        });
    });
}
