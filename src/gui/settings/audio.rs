use super::layout::{label, row, section, settings_grid};
use crate::audio::CHUNK_PERIOD;
use crate::audio::device::{AudioSource, DeviceKind, MappableAvailableDevices};
use crate::settings::audio::SettingsAudio;
use eframe::egui;
use eframe::egui::{Checkbox, RichText, Slider, Ui};

pub(super) fn ui_section_audio(
    ui: &mut Ui,
    settings_audio: &mut SettingsAudio,
    devices: &mut MappableAvailableDevices,
) {
    section(ui, None, |ui| {
        settings_grid("audio_grid").show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                label(ui, "Sources:");
            });
            ui.vertical(|ui| {
                for kind in DeviceKind::ALL {
                    ui.add_space(4.0);
                    ui.label(RichText::new(kind.label()).small().weak());

                    for device in devices.iter(kind) {
                        let source = AudioSource {
                            kind,
                            id: device.id().clone(),
                        };
                        let mut checked = settings_audio.sources.contains(&source);

                        if ui.add(Checkbox::new(&mut checked, device.name())).changed() {
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

                if ui.button("Rescan devices").clicked() {
                    devices.refresh();
                }
            });
            ui.end_row();

            // Stored as a chunk count; people think in seconds.
            let chunk = CHUNK_PERIOD.as_secs_f64();
            row(
                ui,
                "Keep listening:",
                Slider::new(&mut settings_audio.hangover_chunks, 0..=50)
                    .custom_formatter(move |n, _| format!("{:.1} s", n * chunk)),
            )
            .on_hover_text(
                "How long audio keeps flowing after speech stops, so the end of a \
                 phrase is not cut off",
            );
            row(
                ui,
                "Noise gate:",
                Slider::new(&mut settings_audio.vad_threshold, 0..=2000).logarithmic(true),
            )
            .on_hover_text(
                "Sound quieter than this counts as silence and is not recognised. \
                 Raise it if background noise turns into stray words, lower it if \
                 quiet speech is missed.",
            );
        });
    });
}
