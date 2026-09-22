use super::layout::{Squared, label, row, section, settings_grid};
use crate::audio::device::{AudioSource, DeviceKind, MappableAvailableDevices};
use crate::settings::audio::SettingsAudio;
use eframe::egui;
use eframe::egui::{Checkbox, RichText, Slider, Ui};

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

            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                label(ui, "Sources:");
            });
            ui.vertical(|ui| {
                if ui.button("⟳  Rescan devices").clicked() {
                    devices.refresh();
                }

                for kind in DeviceKind::ALL {
                    ui.add_space(4.0);
                    ui.label(RichText::new(kind.label()).small().weak());

                    for device in devices.iter(kind) {
                        let source = AudioSource {
                            kind,
                            id: device.id().clone(),
                        };
                        let mut checked = settings_audio.sources.contains(&source);

                        if ui
                            .add(Squared(Checkbox::new(&mut checked, device.name())))
                            .changed()
                        {
                            if checked {
                                settings_audio.sources.push(source);
                            } else {
                                settings_audio.sources.retain(|saved| saved != &source);
                            }
                        }
                    }
                }

                if settings_audio.sources.is_empty() {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Nothing ticked — system audio will be used")
                            .small()
                            .weak(),
                    );
                }
            });
            ui.end_row();
        });
    });
}
