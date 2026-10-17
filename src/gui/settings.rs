pub(super) mod about;
pub(super) mod appearance;
pub(super) mod audio;
pub(super) mod general;
pub(super) mod layout;
pub(super) mod position;
pub(super) mod provider;

use about::ui_section_about;
use appearance::ui_section_appearance;
use audio::ui_section_audio;
use general::ui_section_general;
use position::ui_section_position;
use provider::ui_section_provider;

use crate::audio::device::MappableAvailableDevices;
use crate::gui::monitor::Monitor;
use crate::gui::state::{PendingState, StateManager};
use crate::gui::title_bar::show_title_bar;
use crate::gui::{Notify, theme};
use crate::settings::provider::ProviderType;
use crate::settings::{SettingsApp, SettingsManager};
use crate::stt::adapters::vosk::probe::VoskProbe;
use eframe::egui::{self, Button, Margin, RichText, ScrollArea, Ui, vec2};
use eframe::epaint::StrokeKind;
use egui_toast::Toasts;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Speech,
    Sources,
    Subtitles,
    App,
}

impl Tab {
    const ALL: [Self; 4] = [Self::Speech, Self::Sources, Self::Subtitles, Self::App];

    fn label(self) -> &'static str {
        match self {
            Self::Speech => "Speech",
            Self::Sources => "Sources",
            Self::Subtitles => "Subtitles",
            Self::App => "App",
        }
    }
}

pub struct SettingsScreen {
    pub devices: MappableAvailableDevices,
    pub vosk_probe: VoskProbe,
    pub monitors: Vec<Monitor>,
    pub tab: Tab,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            devices: MappableAvailableDevices::from_default_host(),
            vosk_probe: VoskProbe::default(),
            monitors: Monitor::list(),
            tab: Tab::default(),
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
    show_title_bar(ui);
    ui_tabs(ui, &mut screen.tab);
    ui_bottom_panel(
        ui,
        settings_manager,
        manager,
        toasts,
        tray_failed,
        &screen.devices,
    );

    egui::CentralPanel::default()
        .frame(egui::Frame::central_panel(&ui.ctx().global_style()).inner_margin(15.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);

            let settings = &mut settings_manager.settings;
            let key_storage = &settings_manager.key_storage;
            let tab = screen.tab;
            ScrollArea::vertical()
                .id_salt(tab.label())
                .show(ui, |ui| match tab {
                    Tab::Speech => ui_section_provider(
                        ui,
                        &mut settings.provider,
                        key_storage,
                        &mut screen.vosk_probe,
                    ),
                    Tab::Sources => ui_section_audio(ui, &mut settings.audio, &mut screen.devices),
                    Tab::Subtitles => {
                        ui_section_position(ui, &mut settings.ui, &mut screen.monitors);
                        ui_section_appearance(ui, &mut settings.ui);
                    }
                    Tab::App => {
                        ui_section_general(ui, &mut settings.general, &mut settings.ui.max_blocks);
                        ui_section_about(ui);
                    }
                });
        });

    window_border(ui);
}

fn ui_tabs(ui: &mut Ui, current: &mut Tab) {
    egui::Panel::top("settings_tabs")
        .frame(
            egui::Frame::new()
                .fill(theme::GROUND)
                .inner_margin(Margin::symmetric(12, 10)),
        )
        .show(ui, |ui| {
            ui.columns(Tab::ALL.len(), |cols| {
                for (col, tab) in cols.iter_mut().zip(Tab::ALL) {
                    col.vertical_centered_justified(|ui| {
                        let mut button = Button::new(tab.label()).min_size(vec2(0.0, 30.0));
                        if *current == tab {
                            button = button.fill(theme::ACCENT);
                        }
                        if ui.add(button).clicked() {
                            *current = tab;
                        }
                    });
                }
            });
        });
}

fn ui_bottom_panel(
    ui: &mut Ui,
    settings_manager: &mut SettingsManager,
    manager: &mut StateManager,
    toasts: &mut Toasts,
    tray_failed: bool,
    devices: &MappableAvailableDevices,
) {
    egui::Panel::bottom("settings_bottom_panel")
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(theme::GROUND)
                .inner_margin(Margin::symmetric(12, 12)),
        )
        .show(ui, |ui| {
            let summary = summary(&settings_manager.settings, devices);
            ui.add(egui::Label::new(RichText::new(summary).small().color(theme::LABEL)).truncate());
            ui.add_space(6.0);

            ui.vertical_centered_justified(|ui| {
                let start_button = Button::new(RichText::new("Start").strong())
                    .min_size(vec2(0.0, 40.0))
                    .fill(theme::ACCENT);
                if ui.add(start_button).clicked() {
                    start(settings_manager, manager, toasts, tray_failed);
                }
            });
        });
}

fn start(
    settings_manager: &mut SettingsManager,
    manager: &mut StateManager,
    toasts: &mut Toasts,
    tray_failed: bool,
) {
    if let Err(e) = settings_manager.save() {
        toasts.error(format!("Failed to save settings: {e}"));
    }

    if tray_failed {
        toasts.warn(
            "Tray is unavailable, there will be nothing to exit the overlay. \
             The launch has been cancelled.",
        );
        return;
    }

    let provider = &settings_manager.settings.provider;
    match provider.active_type {
        ProviderType::Soniox if provider.soniox.api_key.trim().is_empty() => {
            toasts.warn("No API key provided for Soniox!");
        }
        ProviderType::Vosk if provider.vosk.model_path.as_os_str().is_empty() => {
            toasts.warn("No model path provided for Vosk!");
        }
        _ => manager.switch(PendingState::Overlay),
    }
}

fn summary(settings: &SettingsApp, devices: &MappableAvailableDevices) -> String {
    let engine = match settings.provider.active_type {
        ProviderType::Soniox => "Soniox",
        ProviderType::Vosk => "Vosk",
    };
    let native: Vec<_> = settings
        .audio
        .sources
        .iter()
        .filter(|source| source.is_native())
        .collect();
    let audio = match native.as_slice() {
        [] => "default speakers".to_owned(),
        [one] => devices.get(one.kind, &one.id).map_or_else(
            || "a disconnected device".to_owned(),
            |d| d.name().to_owned(),
        ),
        many => format!("{} sources", many.len()),
    };
    format!("{engine} · {audio}")
}

fn window_border(ui: &Ui) {
    let screen = ui.ctx().content_rect();
    ui.ctx()
        .layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("window_border"),
        ))
        .rect_stroke(
            screen,
            0.0,
            egui::Stroke::new(1.0, theme::HOVER),
            StrokeKind::Inside,
        );
}
