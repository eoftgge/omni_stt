use crate::SETTINGS_WINDOW_SIZE;
use display_info::DisplayInfo;
use eframe::egui::{Context, Pos2, ViewportCommand, pos2};

/// A connected monitor, as the settings list shows it.
pub struct Monitor {
    /// The system's name for it (`\\.\DISPLAY2` on Windows). This is what the
    /// config stores: an index would shift whenever a screen is plugged in.
    pub name: String,
    pub label: String,
}

impl Monitor {
    pub fn list() -> Vec<Self> {
        match DisplayInfo::all() {
            Ok(displays) => displays
                .into_iter()
                .map(|display| Self {
                    label: label(&display),
                    name: display.name,
                })
                .collect(),
            Err(e) => {
                tracing::warn!("Failed to list monitors: {e}");
                Vec::new()
            }
        }
    }
}

fn label(display: &DisplayInfo) -> String {
    let primary = if display.is_primary { ", primary" } else { "" };
    format!(
        "{} ({}×{}{primary})",
        display.friendly_name, display.width, display.height
    )
}

/// Centres the window on the named monitor, so that maximizing it next fills
/// that monitor: Windows maximizes onto whichever screen holds most of the
/// window, and centre-on-centre survives rounding that a corner would not.
///
/// Returns where the window was, in physical pixels, for [`restore`]. `None`
/// when the monitor is unplugged, which is ordinary (a laptop away from its
/// projector): the window stays put and the overlay opens where it is.
pub fn move_to(ctx: &Context, name: &str) -> Option<Pos2> {
    // On a restart from the tray the overlay is already maximized on its
    // monitor, and moving a maximized window leaves it half-restored.
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return None;
    }

    let Ok(display) = DisplayInfo::from_name(name) else {
        tracing::info!("Monitor {name} is not connected, keeping the current one");
        return None;
    };

    let ppp = ctx.pixels_per_point();
    let origin = ctx
        .input(|i| i.viewport().outer_rect)
        .map_or(Pos2::ZERO, |rect| rect.min * ppp);

    // `OuterPosition` takes points and egui multiplies them by
    // `pixels_per_point` on the way to winit, which wants physical pixels.
    // display-info reports Windows screens in physical pixels, X11 ones
    // divided by the scale factor, and macOS ones in points, which winit
    // itself scales by the window's native factor.
    let to_physical = if cfg!(windows) {
        1.0
    } else if cfg!(target_os = "macos") {
        ctx.native_pixels_per_point().unwrap_or(1.0)
    } else {
        display.scale_factor
    };
    let center_x = (display.x as f32 + display.width as f32 / 2.0) * to_physical / ppp;
    let center_y = (display.y as f32 + display.height as f32 / 2.0) * to_physical / ppp;

    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(pos2(
        center_x - SETTINGS_WINDOW_SIZE.x / 2.0,
        center_y - SETTINGS_WINDOW_SIZE.y / 2.0,
    )));
    Some(origin)
}

/// Puts the window back where [`move_to`] found it. The origin is kept in
/// physical pixels because the two monitors may not share a scale factor.
pub fn restore(ctx: &Context, origin: Pos2) {
    let ppp = ctx.pixels_per_point();
    ctx.send_viewport_cmd(ViewportCommand::OuterPosition(origin / ppp));
}
