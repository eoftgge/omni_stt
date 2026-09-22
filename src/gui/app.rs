use crate::event::PipelineEvent;
use crate::gui::Notify;
use crate::gui::overlay::draw_subtitles;
use crate::gui::settings::{SettingsScreen, show_settings_window};
use crate::gui::state::{AppState, LoadingOutcome, PendingState, StateManager};
use crate::gui::tray::{AppTray, TrayAction};
use crate::logger::TracingControl;
use crate::pipeline::Pipeline;
use crate::settings::SettingsManager;
use crate::stt::store::TranscriptionStore;
use crate::stt::transcript::TranscriptWriter;
use eframe::App;
use eframe::egui::{
    Align, Area, Color32, Id, Layout, Order, RichText, Ui, ViewportCommand, Visuals, WindowLevel,
};
use egui_toast::{ToastKind, Toasts};
use std::time::Duration;

pub struct SubtitlesApp {
    settings_manager: SettingsManager,
    store: TranscriptionStore,
    toasts: Toasts,
    state_manager: StateManager,
    frame_counter: u64,
    tray: AppTray,
    tracing_control: TracingControl,
    screen: SettingsScreen,
    transcript: Option<TranscriptWriter>,
}

impl SubtitlesApp {
    pub fn new(
        settings_manager: SettingsManager,
        tracing_control: TracingControl,
        tray: AppTray,
    ) -> Self {
        Self {
            store: TranscriptionStore::new(settings_manager.settings.ui.max_blocks),
            toasts: Toasts::default(),
            state_manager: StateManager::new(),
            frame_counter: 0,
            screen: SettingsScreen::new(),
            settings_manager,
            tracing_control,
            tray,
            transcript: None,
        }
    }
}

impl App for SubtitlesApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let state_manager = &mut self.state_manager;

        match self.tray.poll() {
            Some(Ok(action)) => match action {
                TrayAction::OpenSettings => state_manager.switch(PendingState::Settings),
                TrayAction::Restart => state_manager.switch(PendingState::Overlay),
                TrayAction::Quit => ui.ctx().send_viewport_cmd(ViewportCommand::Close),
            },
            Some(Err(e)) => {
                tracing::error!("Tray unavailable: {e}");
                self.tray.set_failed();
                self.toasts
                    .error("Tray unavailable: there will be no way to exit the overlay");
            }
            None => {}
        }

        let settings = &self.settings_manager.settings;
        if let Err(err) = state_manager.resolve(
            ui.ctx(),
            &mut self.store,
            settings,
            &mut self.screen.devices,
        ) {
            self.toasts.error(format!("{err:?}"));
        }

        match state_manager.poll_loading(ui.ctx(), settings.ui.enable_high_priority) {
            Ok(LoadingOutcome::Ready) => self.toasts.info("Starting subtitles overlay..."),
            Ok(_) => {}
            Err(e) => self.toasts.error(e.to_string()),
        }

        match state_manager.app_state_mut() {
            AppState::Settings => show_settings_window(
                ui,
                &mut self.settings_manager,
                state_manager,
                &mut self.toasts,
                self.tray.is_failed(),
                &mut self.screen,
            ),
            AppState::Loading { .. } => {
                let t = ui.ctx().input(|i| i.time);
                let pulse = 0.5 + 0.5 * ((t as f32) * 4.0).sin();
                let alpha = (120.0 + pulse * 135.0) as u8;

                ui.centered_and_justified(|ui| {
                    let dots = (t * 2.0) as usize % 4;
                    let text = format!("Loading model{}", ".".repeat(dots));
                    ui.label(
                        RichText::new(text)
                            .size(20.0)
                            .color(Color32::from_white_alpha(alpha)),
                    );
                });
                ui.ctx().request_repaint();
            }
            AppState::Overlay(service) => {
                let timeout = Duration::from_secs(15);
                self.store.clear_if_silent(timeout);
                if let Some(last_activity) = self.store.last_activity() {
                    let elapsed = last_activity.elapsed();
                    if elapsed < timeout {
                        ui.ctx().request_repaint_after(timeout - elapsed);
                    }
                }

                let ctx = ui.ctx();
                let settings_ui = &settings.ui;
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
                                settings_ui.is_text_outline,
                            );
                        });
                    });

                self.frame_counter += 1;
            }
        }

        let save_transcripts = self.settings_manager.settings.general.save_transcripts;
        sync_transcript(&mut self.transcript, save_transcripts);

        for block in self.store.take_completed() {
            if let Some(writer) = &mut self.transcript {
                writer.write(&block);
            }
        }

        self.toasts.show(ui);
        self.tracing_control
            .sync(&self.settings_manager.settings.general);
    }

    fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
        self.state_manager.color(visuals)
    }
}

impl Drop for SubtitlesApp {
    fn drop(&mut self) {
        self.store.finish();
        for block in self.store.take_completed() {
            if let Some(writer) = &mut self.transcript {
                writer.write(&block);
            }
        }
    }
}

fn sync_transcript(writer: &mut Option<TranscriptWriter>, enabled: bool) {
    match (enabled, writer.is_some()) {
        (true, false) => *writer = Some(TranscriptWriter::open()),
        (false, true) => *writer = None,
        _ => {}
    }
}

fn process_events(service: &mut Pipeline, store: &mut TranscriptionStore, toasts: &mut Toasts) {
    while let Ok(event) = service.receiver.try_recv() {
        match event {
            PipelineEvent::Transcript(data) => {
                store.update(data);
            }
            PipelineEvent::Interim(segments) => store.update_interim(segments),
            PipelineEvent::Warning(msg) => toasts.warn(msg),
            PipelineEvent::Error(err) => toasts.error(err.to_string()),
            PipelineEvent::AudioLost(msg) => toasts.error(msg),
            PipelineEvent::Connected(flag_first_connection) => {
                store.ensure_separator();
                if flag_first_connection {
                    toasts.info("Connected to speech server!");
                }
            }
            PipelineEvent::Disconnected => {
                // deliberately shorter than the warning default: this fires on
                // every reconnect and would be obtrusive at five seconds
                toasts.notify(ToastKind::Warning, 2.0, "Connection lost. Reconnecting...");
            }
        };
    }
}
