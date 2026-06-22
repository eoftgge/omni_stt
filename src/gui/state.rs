use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;
use crate::stt::store::TranscriptionStore;
use crate::transcription::device::MappableAvailableDevices;
use crate::transcription::service::TranscriptionService;
use eframe::egui::{Context, ViewportCommand, Visuals, WindowLevel};

fn apply_overlay_window(ctx: &Context, enable_high_priority: bool) {
    ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
    ctx.send_viewport_cmd(ViewportCommand::Transparent(true));
    ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
    if enable_high_priority {
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
    }
}

fn apply_settings_window(ctx: &Context) {
    ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
    ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
    ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(false));
    ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
}

pub struct StateManager {
    app_state: AppState,
    pending_state: Option<PendingState>,
}

#[derive(Clone, Copy)]
pub enum PendingState {
    Settings,
    Overlay,
}

pub enum AppState {
    Settings,
    Loading {
        rx: tokio::sync::oneshot::Receiver<Result<TranscriptionService, OmniSttErrors>>,
    },
    Overlay(TranscriptionService),
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            app_state: AppState::Settings,
            pending_state: Some(PendingState::Settings),
        }
    }

    pub fn switch(&mut self, new_state: PendingState) {
        self.pending_state = Some(new_state);
    }

    pub fn resolve(
        &mut self,
        ctx: &Context,
        store: &mut TranscriptionStore,
        settings: &SettingsApp,
        devices: &MappableAvailableDevices,
    ) -> Result<(), OmniSttErrors> {
        let Some(resolved) = self.pending_state.take() else {
            return Ok(());
        };

        match resolved {
            PendingState::Settings => {
                resolved.apply_window_state(ctx, settings.ui.enable_high_priority);
                self.app_state = AppState::Settings;
            },
            PendingState::Overlay => {
                store.resize(settings.ui.max_blocks);

                let ctx = ctx.clone();
                let (tx, rx) = tokio::sync::oneshot::channel();

                let settings = settings.clone();
                let ctx_for_service = ctx.clone();
                let device = devices
                    .to_output_device(settings.audio.device_id.as_ref())
                    .ok_or(OmniSttErrors::NotFoundOutputDevice)?;

                tokio::spawn(async move {
                    let result = TranscriptionService::start(
                        &settings,
                        device,
                        move || ctx_for_service.request_repaint(),
                    )
                        .await;
                    let _ = tx.send(result);
                });

                self.app_state = AppState::Loading { rx };
                ctx.request_repaint();
            }
        }
        Ok(())
    }

    pub fn poll_loading(&mut self, ctx: &Context, enable_high_priority: bool) -> Result<(), OmniSttErrors> {
        let AppState::Loading { rx } = &mut self.app_state else {
            return Ok(());
        };

        match rx.try_recv() {
            Ok(Ok(service)) => {
                apply_overlay_window(ctx, enable_high_priority);
                self.app_state = AppState::Overlay(service);
            }
            Ok(Err(e)) => {
                self.switch(PendingState::Settings);
                return Err(e);
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                ctx.request_repaint();
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                self.switch(PendingState::Settings);
                return Err(OmniSttErrors::Internal("Model loading task failed".into()));
            }
        }
        Ok(())
    }

    pub fn app_state(&self) -> &AppState {
        &self.app_state
    }

    pub fn app_state_mut(&mut self) -> &mut AppState {
        &mut self.app_state
    }

    pub fn color(&self, visuals: &Visuals) -> [f32; 4] {
        match self.app_state() {
            AppState::Overlay(_) => [0.0, 0.0, 0.0, 0.0],
            _ => visuals.window_fill().to_normalized_gamma_f32(),
        }
    }
}

impl PendingState {
    pub fn apply_window_state(&self, ctx: &Context, enable_high_priority: bool) {
        match self {
            Self::Settings => apply_settings_window(ctx),
            Self::Overlay => apply_overlay_window(ctx, enable_high_priority),
        }
    }
}
