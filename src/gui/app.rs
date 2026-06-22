use crate::gui::overlay::draw_subtitles;
use crate::gui::settings::show_settings_window;
use crate::gui::state::{AppState, StateManager};
use crate::settings::SettingsManager;
use crate::stt::event::SttEvent;
use crate::stt::store::TranscriptionStore;
use crate::transcription::device::MappableAvailableDevices;
use crate::transcription::service::TranscriptionService;
use eframe::egui::{Align, Area, Id, Layout, Order, Ui, ViewportCommand, Visuals, WindowLevel};
use eframe::{App, Frame};
use egui_notify::Toasts;
use std::time::Duration;
use tracing_appender::non_blocking::WorkerGuard;

fn process_events(
    service: &mut TranscriptionService,
    store: &mut TranscriptionStore,
    toasts: &mut Toasts,
) {
    while let Ok(event) = service.receiver.try_recv() {
        match event {
            SttEvent::Transcript(data) => {
                store.update(data);
            }
            SttEvent::Warning(msg) => {
                toasts
                    .warning(msg)
                    .duration(Duration::from_secs(4))
                    .closable(false);
            }
            SttEvent::Error(err) => {
                toasts
                    .error(err.to_string())
                    .duration(Duration::from_secs(4))
                    .closable(false);
            }
            SttEvent::Connected(flag_first_connection) => {
                store.ensure_separator();
                if flag_first_connection {
                    toasts
                        .info("Connected to speech server!")
                        .duration(Duration::from_secs(4))
                        .closable(false);
                }
            }
            SttEvent::Disconnected => {
                toasts
                    .warning("Connection lost. Reconnecting...")
                    .duration(Duration::from_secs(2))
                    .closable(false);
            }
        };
    }
}

pub struct SubtitlesApp {
    settings_manager: SettingsManager,
    store: TranscriptionStore,
    toasts: Toasts,
    manager: StateManager,
    frame_counter: u64,
    devices: MappableAvailableDevices,
    _guard: Option<WorkerGuard>,
}

impl SubtitlesApp {
    pub fn new(settings_manager: SettingsManager, guard: Option<WorkerGuard>) -> Self {
        Self {
            store: TranscriptionStore::new(settings_manager.settings.ui.max_blocks),
            toasts: Toasts::new(),
            manager: StateManager::new(),
            settings_manager,
            frame_counter: 0,
            devices: MappableAvailableDevices::from_default_host(),
            _guard: guard,
        }
    }
}

impl App for SubtitlesApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        if let Err(err) = self.manager.resolve(
            ui.ctx(),
            &mut self.store,
            &self.settings_manager.settings,
            &self.devices,
        ) {
            self.toasts.error(format!("{:?}", err)).closable(false);
        }
        let manager = &mut self.manager;

        match manager.app_state_mut() {
            AppState::Settings => show_settings_window(
                ui,
                &mut self.settings_manager,
                manager,
                &mut self.toasts,
                &mut self.devices,
            ),
            AppState::Overlay(service) => {
                let timeout = Duration::from_secs(15);
                self.store.clear_if_silent(timeout);
                self.store.schedule(ui.ctx().clone(), timeout);

                let ctx = ui.ctx();
                let settings_ui = &self.settings_manager.settings.ui;
                process_events(service, &mut self.store, &mut self.toasts);
                if settings_ui.enable_high_priority && self.frame_counter >= 100 {
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
                    self.frame_counter = 0;
                }
                let (anchor, offset) = settings_ui.get_anchor();
                Area::new(Id::from("subtitles_area"))
                    .anchor(anchor, offset)
                    .order(Order::Foreground)
                    .show(ctx, |ui| {
                        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                            draw_subtitles(
                                ui,
                                &self.store,
                                settings_ui.font_size as f32,
                                settings_ui.text_color(),
                                settings_ui.background_color(),
                            );
                        });
                    });

                self.frame_counter += 1;
            }
        }

        self.toasts.show(ui.ctx());
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }
}
