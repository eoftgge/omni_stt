pub(crate) mod color;
pub(crate) mod outline;

use crate::gui::overlay::color::get_interim_color;
use crate::gui::overlay::outline::{TextOutline, add_outlined_text};
use crate::subtitles::store::VisualReplica;
use crate::subtitles::store::{TranscriptionStore, prepare_replicas};
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
    is_text_outline: bool,
    max_lines: usize,
) {
    let text_outline = is_text_outline.then(|| TextOutline::for_font_size(font_size));
    let replicas = prepare_replicas(store);
    if replicas.is_empty() {
        return;
    }

    let screen_width = ui.ctx().content_rect().width();
    let max_width = (screen_width * 0.8).min(1200.0);
    let interim_color = get_interim_color(text_color);

    let id = ui.id().with("subtitles_anim_box");
    let inner = Frame::new()
        .fill(Color32::TRANSPARENT)
        .corner_radius(12.0)
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_max_width(max_width);
            ui.vertical(|ui| {
                let pad = text_outline.map_or(0.0, |o| o.width);
                let wrap_width = ui.available_width() - pad * 2.0;
                let plan = plan_visible_lines(ui, &replicas, font_size, wrap_width, max_lines);

                for (replica, skip) in plan {
                    draw_replica_row(
                        ui,
                        replica,
                        font_size,
                        wrap_width,
                        text_color,
                        interim_color,
                        text_outline,
                        skip,
                    );
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
}

fn draw_replica_row(
    ui: &mut Ui,
    replica: &VisualReplica,
    font_size: f32,
    wrap_width: f32,
    text_color: Color32,
    interim_color: Color32,
    outline: Option<TextOutline>,
    skip: usize,
) {
    add_outlined_text(ui, outline, text_color, |override_color| {
        build_job(
            replica,
            font_size,
            wrap_width,
            skip,
            |is_interim| match override_color {
                Some(color) => color,
                None if is_interim => interim_color,
                None => text_color,
            },
        )
    });
}

fn plan_visible_lines<'a>(
    ui: &Ui,
    replicas: &'a [VisualReplica<'a>],
    font_size: f32,
    wrap_width: f32,
    max_lines: usize,
) -> Vec<(&'a VisualReplica<'a>, usize)> {
    let mut budget = max_lines.max(1);
    let mut plan = Vec::new();

    for replica in replicas.iter().rev() {
        if budget == 0 {
            break;
        }

        // Color has no effect on layout, so one measuring pass in any color
        // predicts what all nine outline passes will produce. egui caches
        // galleys by job, so on a frame where nothing changed this is free.
        let job = build_job(replica, font_size, wrap_width, 0, |_| Color32::WHITE);
        let galley = ui.painter().layout_job(job);
        let rows = galley.rows.len();

        if rows <= budget {
            budget -= rows;
            plan.push((replica, 0));
            continue;
        }

        // Only part of this replica fits. Drop whole rows off its front,
        // counting characters with egui's own per-row accounting: wrapping
        // moves the space it breaks on and sometimes swallows it, so counting
        // the source text by hand drifts and eventually cuts mid-word.
        let skip = galley.rows[..rows - budget]
            .iter()
            .map(|row| row.char_count_including_newline().0)
            .sum();

        plan.push((replica, skip));
        budget = 0;
    }

    plan.reverse();
    plan
}

fn build_job(
    replica: &VisualReplica,
    font_size: f32,
    wrap_width: f32,
    skip: usize,
    color_for: impl Fn(bool) -> Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.break_anywhere = false;
    job.wrap.max_width = wrap_width;

    let font_id = FontId::proportional(font_size);
    let mut remaining = skip;

    // Resuming mid-paragraph, the piece we land in usually opens with the space
    // that used to separate it from the dropped text. Pretending something
    // ending in a space came before makes the existing trim swallow it.
    let mut last_ends_with_space = skip > 0;

    if let Some(id) = &replica.speaker {
        let prefix = format!("{id}: ");
        if let Some(text) = advance(&prefix, &mut remaining) {
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: color_for(false),
                    ..Default::default()
                },
            );
            last_ends_with_space = text.ends_with(' ');
        }
    }

    for elem in replica.elements.iter() {
        let Some(mut text) = advance(elem.text, &mut remaining) else {
            continue;
        };
        if last_ends_with_space && text.starts_with(' ') {
            text = text.trim_start();
        }

        job.append(
            text,
            0.0,
            TextFormat {
                font_id: font_id.clone(),
                color: color_for(elem.is_interim),
                ..Default::default()
            },
        );
        last_ends_with_space = text.ends_with(' ');
    }

    job
}

/// Eats up to `remaining` characters off the front of `text`, returning what is
/// left, or `None` when the whole piece was eaten.
fn advance<'a>(text: &'a str, remaining: &mut usize) -> Option<&'a str> {
    if *remaining == 0 {
        return Some(text);
    }

    match text.char_indices().nth(*remaining) {
        Some((byte, _)) => {
            *remaining = 0;
            Some(&text[byte..])
        }
        None => {
            *remaining -= text.chars().count();
            None
        }
    }
}
