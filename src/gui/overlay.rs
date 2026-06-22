use crate::gui::color::get_interim_color;
use crate::stt::store::TranscriptionStore;
use crate::transcription::replicas::{VisualReplica, prepare_replicas};
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, Frame, LayerId, Order, Rect, Stroke, TextFormat, Ui, Vec2};
use eframe::epaint::StrokeKind;

const ANIM_TIME: f32 = 0.08;

pub fn draw_subtitles(
    ui: &mut Ui,
    store: &TranscriptionStore,
    font_size: f32,
    text_color: Color32,
    background_color: Color32,
) {
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
                    draw_replica_row(ui, replica, font_size, text_color, interim_color);
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
) {
    let mut job = LayoutJob::default();
    let mut last_ends_with_space = false;
    job.wrap.break_anywhere = false;
    if let Some(id) = &replica.speaker {
        let speaker_format = TextFormat {
            font_id: FontId::proportional(font_size),
            color: text_color,
            ..Default::default()
        };
        job.append(id, 0.0, speaker_format.clone());
        job.append(": ", 0.0, speaker_format);
        last_ends_with_space = true;
    }

    for elem in replica.elements.iter() {
        let format = TextFormat {
            font_id: FontId::proportional(font_size),
            color: if elem.is_interim {
                interim_color
            } else {
                text_color
            },
            ..Default::default()
        };
        let mut text = elem.text;
        if last_ends_with_space && text.starts_with(' ') {
            text = text.trim_start();
        }

        job.append(text, 0.0, format);
        last_ends_with_space = text.ends_with(' ');
    }

    ui.label(job);
}
