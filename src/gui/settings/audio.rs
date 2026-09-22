use super::layout::{label, row, section, settings_grid};
use crate::settings::SettingsAudio;
use crate::transcription::device::{DeviceKind, MappableAvailableDevices};
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

            label(ui, "Source:");
            let previous_kind = settings_audio.device_kind;
            ui.horizontal(|ui| {
                for kind in DeviceKind::ALL {
                    ui.selectable_value(&mut settings_audio.device_kind, kind, kind.label());
                }
            });
            ui.end_row();

            // A saved id lives in one of the two lists only, so after a switch
            // it can never match again. Drop it and fall back to the default
            // device of the kind just chosen.
            if settings_audio.device_kind != previous_kind {
                settings_audio.device_id = None;
            }

            label(ui, "Device:");
            let kind = settings_audio.device_kind;
            let default_label = "System Default";
            let current = settings_audio
                .device_id()
                .and_then(|id| devices.get(kind, &id))
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
                    for device in devices.iter(kind) {
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
