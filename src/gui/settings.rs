pub(super) mod appearance;
pub(super) mod audio;
pub(super) mod general;
pub(super) mod layout;
pub(super) mod position;
pub(super) mod provider;

use appearance::ui_section_appearance;
use audio::ui_section_audio;
use general::ui_section_general;
use position::ui_section_position;
use provider::ui_section_provider;

use crate::audio::device::MappableAvailableDevices;
use crate::gui::state::{PendingState, StateManager};
use crate::gui::{Notify, theme};
use crate::settings::SettingsManager;
use crate::stt::adapters::types::ProviderType;
use crate::stt::adapters::vosk::probe::VoskProbe;
use eframe::egui::{self, Button, ScrollArea, Ui, vec2};
use egui_toast::Toasts;

pub struct SettingsScreen {
    pub devices: MappableAvailableDevices,
    pub vosk_probe: VoskProbe,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            devices: MappableAvailableDevices::from_default_host(),
            vosk_probe: VoskProbe::default(),
        }
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}

pub fn show_settings_window(
    ui: &mut Ui,
    settings_manager: &mut SettingsManager,
    manager: &mut StateManager,
    toasts: &mut Toasts,
    tray_failed: bool,
    screen: &mut SettingsScreen,
) {
    ui_bottom_panel(ui, settings_manager, manager, toasts, tray_failed);

    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ui.ctx().global_style()).inner_margin(15.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);
            ui.heading("Settings");
            ui.separator();

            let settings = &mut settings_manager.settings;
            let key_storage = &settings_manager.key_storage;
            ScrollArea::vertical().show(ui, |ui| {
                ui_section_general(ui, &mut settings.general);
                ui_section_audio(ui, &mut settings.audio, &mut screen.devices);
                ui_section_provider(
                    ui,
                    &mut settings.provider,
                    key_storage,
                    &mut screen.vosk_probe,
                );
                ui_section_position(ui, &mut settings.ui);
                ui_section_appearance(ui, &mut settings.ui);
                ui.allocate_space(vec2(0.0, 60.0));
            });
        });
}

fn ui_bottom_panel(
    ui: &mut Ui,
    settings_manager: &mut SettingsManager,
    manager: &mut StateManager,
    toasts: &mut Toasts,
    tray_failed: bool,
) {
    egui::Panel::bottom("settings_bottom_panel")
        .resizable(false)
        .min_size(60.0)
        .show(ui, |ui| {
            ui.add_space(15.0);
            ui.columns(2, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    if ui
                        .add(Button::new("💾 Save").min_size(vec2(0.0, 40.0)))
                        .clicked()
                    {
                        match settings_manager.save() {
                            Ok(_) => toasts.success("Settings saved successfully!"),
                            Err(e) => toasts.error(format!("Failed to save: {e}")),
                        }
                    }
                });

                cols[1].vertical_centered_justified(|ui| {
                    let start = Button::new("🚀 Start")
                        .min_size(vec2(0.0, 40.0))
                        .fill(theme::ACCENT);
                    if ui.add(start).clicked() {
                        if tray_failed {
                            toasts.warn(
                                "Tray is unavailable, there will be nothing to exit the overlay. \
                                 The launch has been cancelled.",
                            );
                            return;
                        }

                        let settings_provider = &settings_manager.settings.provider;
                        match settings_provider.active_type {
                            ProviderType::Soniox
                                if settings_provider.soniox.api_key.trim().is_empty() =>
                            {
                                toasts.warn("No API key provided for Soniox!");
                            }
                            ProviderType::Vosk
                                if settings_provider.vosk.model_path.as_os_str().is_empty() =>
                            {
                                toasts.warn("No model path provided for Vosk!");
                            }
                            _ => {
                                manager.switch(PendingState::Overlay);
                            }
                        }
                    }
                });
            });
            ui.add_space(10.0);
        });
}
