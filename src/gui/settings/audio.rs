use crate::gui::settings::layout::{row, section, settings_grid};
use crate::settings::SettingsAudio;
use crate::transcription::device::MappableAvailableDevices;
use eframe::egui::{ComboBox, Slider, Ui};

pub(super) fn ui_section_audio(
    ui: &mut Ui,
    settings_audio: &mut SettingsAudio,
    devices: &mut MappableAvailableDevices,
) {
    section(ui, "Audio", false, |ui| {
        settings_grid("audio_grid").show(ui, |ui| {
            row(
                ui,
                "Hangover Chunks:",
                Slider::new(&mut settings_audio.hangover_chunks, 0..=50),
            );
            row(
                ui,
                "Threshold:",
                Slider::new(&mut settings_audio.vad_threshold, 0..=2000).logarithmic(true),
            );

            ui.label("Output Device:");
            let default_label = "System Default";
            let current = settings_audio
                .device_id()
                .and_then(|d| devices.get(&d))
                .map(|d| d.name())
                .unwrap_or(default_label);

            let mut want_refresh = false;
            ComboBox::from_id_salt("selector_device")
                .selected_text(current)
                .width(100.0)
                .show_ui(ui, |ui| {
                    if ui.button("⟳  Rescan devices").clicked() {
                        want_refresh = true;
                    }
                    ui.separator();
                    ui.selectable_value(&mut settings_audio.device_id, None, default_label);
                    for device in devices.iter() {
                        ui.selectable_value(
                            &mut settings_audio.device_id,
                            Some(device.id().clone()),
                            device.name(),
                        );
                    }
                });

            if want_refresh {
                devices.refresh();
            }
        });
    });
}
