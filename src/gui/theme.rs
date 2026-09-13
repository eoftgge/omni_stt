use eframe::egui::{Color32, Context, CornerRadius, Stroke, Visuals};

pub const GROUND: Color32 = Color32::from_rgb(0x14, 0x16, 0x1F);
pub const SURFACE: Color32 = Color32::from_rgb(0x1E, 0x20, 0x30);
pub const HOVER: Color32 = Color32::from_rgb(0x2A, 0x2D, 0x42);
pub const ACCENT: Color32 = Color32::from_rgb(0x4A, 0x50, 0xC8);
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x5B, 0x62, 0xE8);
pub const TEXT: Color32 = Color32::from_rgb(0xE4, 0xE6, 0xF0);
pub const LABEL: Color32 = Color32::from_rgb(0xB4, 0xBA, 0xD0);
pub const FIELD: Color32 = Color32::from_rgb(0x0E, 0x10, 0x17);

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = GROUND;
    visuals.window_fill = GROUND;
    visuals.extreme_bg_color = FIELD;
    visuals.faint_bg_color = SURFACE;
    visuals.hyperlink_color = ACCENT_LIGHT;

    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, TEXT);

    let radius = CornerRadius::same(6);
    let border = Stroke::new(1.0, HOVER);
    visuals.window_stroke = border;

    let w = &mut visuals.widgets;

    w.noninteractive.bg_fill = GROUND;
    w.noninteractive.weak_bg_fill = GROUND;
    w.noninteractive.bg_stroke = border;
    w.noninteractive.fg_stroke = Stroke::new(1.0, LABEL);
    w.noninteractive.corner_radius = radius;

    w.inactive.bg_fill = SURFACE;
    w.inactive.weak_bg_fill = SURFACE;
    w.inactive.bg_stroke = Stroke::NONE;
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    w.inactive.corner_radius = radius;

    w.hovered.bg_fill = HOVER;
    w.hovered.weak_bg_fill = HOVER;
    w.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    w.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    w.hovered.corner_radius = radius;

    w.active.bg_fill = ACCENT;
    w.active.weak_bg_fill = ACCENT;
    w.active.bg_stroke = Stroke::new(1.0, ACCENT_LIGHT);
    w.active.fg_stroke = Stroke::new(1.0, TEXT);
    w.active.corner_radius = radius;

    w.open.bg_fill = SURFACE;
    w.open.weak_bg_fill = SURFACE;
    w.open.bg_stroke = border;
    w.open.fg_stroke = Stroke::new(1.0, TEXT);
    w.open.corner_radius = radius;

    ctx.set_visuals(visuals);
}
