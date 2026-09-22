use super::layout::{Squared, label, row, section, settings_grid};
use crate::settings::SettingsAudio;
use crate::transcription::device::{DeviceKind, MappableAvailableDevices, SettingDeviceId};
use eframe::egui::{Checkbox, ComboBox, Slider, Ui};

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
            ui_device_picker(
                ui,
                "selector_device",
                &mut settings_audio.device_kind,
                &mut settings_audio.device_id,
                devices,
            );

            row(
                ui,
                "Second source:",
                Squared(Checkbox::without_text(&mut settings_audio.enable_secondary)),
            )
                .on_hover_text("Mix a second capture in — your microphone alongside system audio");

            if settings_audio.enable_secondary {
                label(ui, "Source:");
                ui_device_picker(
                    ui,
                    "selector_device_secondary",
                    &mut settings_audio.secondary_kind,
                    &mut settings_audio.secondary_id,
                    devices,
                );
            }
        });
    });
}

/// Two grid rows: the kind selector, then the device list filtered by it.
///
/// Both sources go through here, so `id_salt` has to differ between the calls —
/// two combo boxes sharing an id would share their popup state as well.
fn ui_device_picker(
    ui: &mut Ui,
    id_salt: &str,
    kind: &mut DeviceKind,
    selected: &mut Option<SettingDeviceId>,
    devices: &mut MappableAvailableDevices,
) {
    let previous_kind = *kind;
    ui.horizontal(|ui| {
        for candidate in DeviceKind::ALL {
            ui.selectable_value(kind, candidate, candidate.label());
        }
    });
    ui.end_row();

    // A saved id lives in one of the two lists only, so after a switch it can
    // never match again. Drop it and fall back to the new kind's default device.
    if *kind != previous_kind {
        *selected = None;
    }

    label(ui, "Device:");

    let default_label = "System Default";
    let current = selected
        .clone()
        .and_then(|id| devices.get(*kind, &id))
        .map(|device| device.name())
        .unwrap_or(default_label);

    let mut want_refresh = false;
    ComboBox::from_id_salt(id_salt)
        .selected_text(current)
        .width(100.0)
        .show_ui(ui, |ui| {
            if ui.button("⟳  Rescan devices").clicked() {
                want_refresh = true;
            }
            ui.separator();
            ui.selectable_value(selected, None, default_label);
            for device in devices.iter(*kind) {
                ui.selectable_value(selected, Some(device.id().clone()), device.name());
            }
        });

    if want_refresh {
        devices.refresh();
    }
    ui.end_row();
}
