use crate::TOOLTIP;
use crate::gui::theme;
use eframe::egui::{
    self, Align2, Color32, FontId, Frame, Id, PointerButton, Response, Sense, Stroke, Ui,
    ViewportCommand, vec2,
};

const HEIGHT: f32 = 34.0;
const BUTTON_WIDTH: f32 = 46.0;
/// Windows' own hover red, so the close button reads as "close" at a glance.
const CLOSE_HOVER: Color32 = Color32::from_rgb(0xC4, 0x2B, 0x1C);

#[derive(Clone, Copy)]
enum Glyph {
    Minimize,
    Close,
}

pub fn show_title_bar(ui: &mut Ui) {
    egui::Panel::top("title_bar")
        .exact_size(HEIGHT)
        .frame(Frame::new().fill(theme::SURFACE))
        .show_separator_line(false)
        .show(ui, |ui| {
            let bar = ui.max_rect();
            let drag = ui.interact(bar, Id::new("title_bar_drag"), Sense::click_and_drag());
            if drag.drag_started_by(PointerButton::Primary) {
                ui.send_viewport_cmd(ViewportCommand::StartDrag);
            }

            ui.painter().text(
                bar.left_center() + vec2(14.0, 0.0),
                Align2::LEFT_CENTER,
                TOOLTIP,
                FontId::proportional(14.0),
                theme::TEXT,
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if window_button(ui, Glyph::Close).clicked() {
                    ui.send_viewport_cmd(ViewportCommand::Close);
                }
                if window_button(ui, Glyph::Minimize).clicked() {
                    ui.send_viewport_cmd(ViewportCommand::Minimized(true));
                }
            });
        });
}

fn window_button(ui: &mut Ui, glyph: Glyph) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(BUTTON_WIDTH, HEIGHT), Sense::click());
    let hovered = response.hovered();

    let fill = match glyph {
        Glyph::Close if hovered => CLOSE_HOVER,
        _ if hovered => theme::HOVER,
        _ => Color32::TRANSPARENT,
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, fill);

    let stroke = Stroke::new(
        1.0,
        if hovered {
            Color32::WHITE
        } else {
            theme::LABEL
        },
    );
    let c = rect.center();
    let r = 5.0;
    match glyph {
        Glyph::Minimize => {
            painter.line_segment([c - vec2(r, 0.0), c + vec2(r, 0.0)], stroke);
        }
        Glyph::Close => {
            painter.line_segment([c - vec2(r, r), c + vec2(r, r)], stroke);
            painter.line_segment([c + vec2(-r, r), c + vec2(r, -r)], stroke);
        }
    }

    response
}
