use crate::SETTINGS_WINDOW_SIZE;
use crate::audio::device::MappableAvailableDevices;
use crate::errors::OmniSttErrors;
use crate::gui::monitor;
use crate::pipeline::Pipeline;
use crate::settings::SettingsApp;
use crate::settings::ui::SettingsUI;
use crate::subtitles::store::TranscriptionStore;
use eframe::egui::{Context, Pos2, ViewportCommand, Visuals, WindowLevel};

/// Windows keeps a per-window GDI redirection surface alongside the real
/// composition, and resizing a layered window from small-and-opaque to
/// maximized-and-transparent leaves the old bitmap in it forever. The
/// desktop never shows it, but GDI-path screen capture does — a white
/// rectangle the size of the settings window. Hiding the window makes the
/// compositor drop that surface, the same way minimizing does.
///
/// Only with DX12 or Vulkan. A GL window never becomes visible again after being hidden, and GL
/// presents through the redirection surface itself, so it has nothing stale.
fn apply_overlay_window(
    ctx: &Context,
    settings: &SettingsUI,
    hide_during_restyle: bool,
) -> Option<Pos2> {
    if hide_during_restyle {
        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
    }

    let origin = settings
        .monitor
        .as_deref()
        .and_then(|name| monitor::move_to(ctx, name));

    ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
    ctx.send_viewport_cmd(ViewportCommand::Transparent(true));
    ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(true));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
    if settings.enable_high_priority {
        ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
    }
    if hide_during_restyle {
        ctx.send_viewport_cmd(ViewportCommand::Visible(true));
    }

    origin
}

fn apply_settings_window(ctx: &Context) {
    ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
    ctx.send_viewport_cmd(ViewportCommand::Transparent(false));
    ctx.send_viewport_cmd(ViewportCommand::MousePassthrough(false));
    ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::Normal));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(false));
    ctx.send_viewport_cmd(ViewportCommand::InnerSize(SETTINGS_WINDOW_SIZE));
}

pub struct StateManager {
    app_state: AppState,
    pending_state: Option<PendingState>,
    hide_during_restyle: bool,
    /// Where the settings window stood before the overlay moved to another
    /// monitor, in physical pixels; put back on the way out.
    settings_origin: Option<Pos2>,
}

pub enum LoadingOutcome {
    Pending,
    Ready,
    Idle,
}

#[derive(Clone, Copy)]
pub enum PendingState {
    Settings,
    Overlay,
}

pub enum AppState {
    Settings,
    Loading {
        rx: tokio::sync::oneshot::Receiver<Result<Pipeline, OmniSttErrors>>,
    },
    Overlay(Pipeline),
}

impl StateManager {
    pub fn new(hide_during_restyle: bool) -> Self {
        Self {
            hide_during_restyle,
            app_state: AppState::Settings,
            pending_state: Some(PendingState::Settings),
            settings_origin: None,
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
        devices: &mut MappableAvailableDevices,
    ) -> Result<(), OmniSttErrors> {
        let Some(resolved) = self.pending_state.take() else {
            return Ok(());
        };

        match resolved {
            PendingState::Settings => {
                store.finish();
                devices.refresh();
                apply_settings_window(ctx);
                if let Some(origin) = self.settings_origin.take() {
                    monitor::restore(ctx, origin);
                }
                self.app_state = AppState::Settings;
            }
            PendingState::Overlay => {
                store.resize(settings.ui.max_blocks);

                let ctx = ctx.clone();
                let (tx, rx) = tokio::sync::oneshot::channel();

                let settings = settings.clone();
                let ctx_for_service = ctx.clone();
                let devices_to_open = devices.resolve(&settings.audio.sources);
                if devices_to_open.is_empty() {
                    return Err(OmniSttErrors::NotFoundAudioDevice);
                }

                tokio::spawn(async move {
                    let result = Pipeline::start(&settings, devices_to_open, move || {
                        ctx_for_service.request_repaint()
                    })
                    .await;
                    let _ = tx.send(result);
                });

                self.app_state = AppState::Loading { rx };
                ctx.request_repaint();
            }
        }
        Ok(())
    }

    pub fn poll_loading(
        &mut self,
        ctx: &Context,
        settings: &SettingsUI,
    ) -> Result<LoadingOutcome, OmniSttErrors> {
        let AppState::Loading { rx } = &mut self.app_state else {
            return Ok(LoadingOutcome::Idle);
        };

        match rx.try_recv() {
            Ok(Ok(service)) => {
                let origin = apply_overlay_window(ctx, settings, self.hide_during_restyle);
                // A restart from the tray passes through here again. Keep the
                // first origin, the one that is actually the settings window's.
                self.settings_origin = self.settings_origin.or(origin);
                self.app_state = AppState::Overlay(service);
                Ok(LoadingOutcome::Ready)
            }
            Ok(Err(e)) => {
                self.switch(PendingState::Settings);
                Err(e)
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                ctx.request_repaint();
                Ok(LoadingOutcome::Pending)
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                self.switch(PendingState::Settings);
                Err(OmniSttErrors::Internal("Model loading task failed".into()))
            }
        }
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
