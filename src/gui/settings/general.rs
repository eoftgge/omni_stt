use super::layout::{Squared, label, row, section, settings_grid};
use crate::logger::LEVELS;
use crate::settings::general::SettingsGeneral;
use eframe::egui::{Checkbox, ComboBox, Ui};

pub(super) fn ui_section_general(ui: &mut Ui, settings_general: &mut SettingsGeneral) {
    section(ui, "General", false, |ui| {
        settings_grid("general_grid").show(ui, |ui| {
            label(ui, "Log Level:");
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
                Squared(Checkbox::without_text(&mut settings_general.log_to_file)),
            )
            .on_hover_text("Save logs to a .log file in the app directory");

            row(
                ui,
                "Save transcripts:",
                Squared(Checkbox::without_text(
                    &mut settings_general.save_transcripts,
                )),
            )
            .on_hover_text("Append finished subtitles to transcripts/omni-YYYY-MM-DD.txt");
        });
    });
}
