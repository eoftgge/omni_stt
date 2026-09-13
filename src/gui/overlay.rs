pub(crate) mod outline;
pub(crate) mod color;

use crate::stt::store::TranscriptionStore;
use crate::transcription::replicas::{VisualReplica, prepare_replicas};
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, Frame, LayerId, Order, Rect, Sense, Stroke, TextFormat, Ui, Vec2};
use eframe::epaint::StrokeKind;
use crate::gui::overlay::color::get_interim_color;
use crate::gui::overlay::outline::TextOutline;

const ANIM_TIME: f32 = 0.08;

pub fn draw_subtitles(
    ui: &mut Ui,
    store: &TranscriptionStore,
    font_size: f32,
    text_color: Color32,
    background_color: Color32,
    is_text_outline: bool,
) {
    let text_outline = is_text_outline.then(|| TextOutline::for_font_size(font_size));
    let replicas = prepare_replicas(store);
    if replicas.is_empty() {
        return;
    }

    let max_visual_replicas = store.max_blocks();
    let total_count = replicas.len();
    let start_index = total_count.saturating_sub(max_visual_replicas);
    let visible_replicas = replicas.iter().skip(start_index);

    let screen_width = ui.ctx().content_rect().width();
    let max_width = (screen_width * 0.8).min(1200.0);
    let interim_color = get_interim_color(text_color);

    let id = ui.id().with("subtitles_anim_box");
    let last_target_size = ui.data(|d| d.get_temp::<Vec2>(id)).unwrap_or(Vec2::ZERO);
    let anim_w = ui
        .ctx()
        .animate_value_with_time(id.with("w"), last_target_size.x, ANIM_TIME);
    let anim_h = ui
        .ctx()
        .animate_value_with_time(id.with("h"), last_target_size.y, ANIM_TIME);
    let current_animated_size = Vec2::new(anim_w, anim_h);

    let inner = Frame::new()
        .fill(Color32::TRANSPARENT)
        .corner_radius(12.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_max_width(max_width);
            ui.vertical(|ui| {
                for replica in visible_replicas {
                    draw_replica_row(ui, replica, font_size, text_color, interim_color, text_outline);
                    ui.add_space(4.0);
                }
            });
        });

    let target_rect = inner.response.rect;
    let target_size = target_rect.size();

    if background_color != Color32::TRANSPARENT {
        let anim_w = ui
            .ctx()
            .animate_value_with_time(id.with("w"), target_size.x, ANIM_TIME)
            .max(target_size.x);
        let anim_h = ui
            .ctx()
            .animate_value_with_time(id.with("h"), target_size.y, ANIM_TIME)
            .max(target_size.y);
        let animated_size = Vec2::new(anim_w, anim_h);

        let animated_rect = Rect::from_center_size(target_rect.center(), animated_size);
        let painter = ui.painter().clone();
        painter
            .with_layer_id(LayerId::new(Order::Background, id))
            .rect(
                animated_rect,
                12.0,
                background_color,
                Stroke::NONE,
                StrokeKind::Middle,
            );

        if (animated_size - target_size).length_sq() > 1.0 {
            ui.ctx().request_repaint();
        }
    }

    ui.data_mut(|d| d.insert_temp(id, target_size));
    if (current_animated_size - target_size).length_sq() > 1.0 {
        ui.ctx().request_repaint();
    }
}

fn draw_replica_row(
    ui: &mut Ui,
    replica: &VisualReplica,
    font_size: f32,
    text_color: Color32,
    interim_color: Color32,
    outline: Option<TextOutline>,
) {
    let wrap_width = ui.available_width();

    let main_job = build_job(replica, font_size, wrap_width, |is_interim| {
        if is_interim { interim_color } else { text_color }
    });
    let main = ui.fonts_mut(|f| f.layout_job(main_job));

    let pad = outline.map_or(0.0, |o| o.width);
    let (rect, _) = ui.allocate_exact_size(
        main.size() + Vec2::splat(pad * 2.0),
        Sense::hover(),
    );
    let pos = rect.min + Vec2::splat(pad);

    if let Some(outline) = outline {
        let shadow_job = build_job(replica, font_size, wrap_width, |_| outline.color);
        let shadow = ui.fonts_mut(|f| f.layout_job(shadow_job));
        for offset in outline.offsets() {
            ui.painter().galley(pos + offset, shadow.clone(), outline.color);
        }
    }

    ui.painter().galley(pos, main, text_color);
}

fn build_job(
    replica: &VisualReplica,
    font_size: f32,
    wrap_width: f32,
    color_for: impl Fn(bool) -> Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.break_anywhere = false;
    job.wrap.max_width = wrap_width;

    let font_id = FontId::proportional(font_size);
    let mut last_ends_with_space = false;

    if let Some(id) = &replica.speaker {
        let format = TextFormat {
            font_id: font_id.clone(),
            color: color_for(false),
            ..Default::default()
        };
        job.append(id, 0.0, format.clone());
        job.append(": ", 0.0, format);
        last_ends_with_space = true;
    }

    for elem in replica.elements.iter() {
        let format = TextFormat {
            font_id: font_id.clone(),
            color: color_for(elem.is_interim),
            ..Default::default()
        };
        let mut text = elem.text;
        if last_ends_with_space && text.starts_with(' ') {
            text = text.trim_start();
        }
        job.append(text, 0.0, format);
        last_ends_with_space = text.ends_with(' ');
    }

    job
}

