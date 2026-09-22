use super::layout::{Squared, label, row, section, settings_grid};
use crate::gui::overlay::outline::{TextOutline, add_outlined_text};
use crate::settings::SettingsUI;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Checkbox, FontId, Slider, TextFormat, Ui};

pub(super) fn ui_section_appearance(ui: &mut Ui, settings_ui: &mut SettingsUI) {
    section(ui, "Appearance", false, |ui| {
        settings_grid("appearance_grid").show(ui, |ui| {
            row(
                ui,
                "Max Blocks:",
                Slider::new(&mut settings_ui.max_blocks, 1..=10),
            );
            row(
                ui,
                "Font Size:",
                Slider::new(&mut settings_ui.font_size, 10..=80),
            );
            row(
                ui,
                "Always On Top:",
                Squared(Checkbox::without_text(
                    &mut settings_ui.enable_high_priority,
                )),
            );
            row(
                ui,
                "Text Outline:",
                Squared(Checkbox::without_text(&mut settings_ui.is_text_outline)),
            );
        });

        ui.separator();

        settings_grid("color_grid").show(ui, |ui| {
            label(ui, "Background Color:");
            ui.horizontal(|ui| {
                let color = &mut settings_ui.background_color;
                if ui.color_edit_button_srgba_unmultiplied(color).changed() {
                    settings_ui.background_color = [color[0], color[1], color[2], color[3]];
                }
                if ui.button("Clear").clicked() {
                    settings_ui.background_color = SettingsUI::DEFAULT_BACKGROUND_COLOR;
                }
            });
            ui.end_row();

            label(ui, "Text Color:");
            ui.horizontal(|ui| {
                let color = &mut settings_ui.text_color;
                if ui.color_edit_button_srgb(color).changed() {
                    settings_ui.text_color = [color[0], color[1], color[2]];
                }
                if ui
                    .button("Clear")
                    .on_hover_text("Reset to default")
                    .clicked()
                {
                    settings_ui.text_color = SettingsUI::DEFAULT_TEXT_COLOR; // yellow
                }
            });
            ui.end_row();
        });

        ui_preview(ui, settings_ui);
    });
}

fn ui_preview(ui: &mut Ui, settings_ui: &SettingsUI) {
    let font_size = settings_ui.font_size as f32;
    let text_color = settings_ui.text_color();
    let outline = settings_ui
        .is_text_outline
        .then(|| TextOutline::for_font_size(font_size));

    egui::Frame::new()
        .fill(settings_ui.background_color())
        .corner_radius(5.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            let text = format!("Preview ({font_size:.0}px)");
            add_outlined_text(ui, outline, text_color, |override_color| {
                let mut job = LayoutJob::default();
                job.append(
                    &text,
                    0.0,
                    TextFormat {
                        font_id: FontId::proportional(font_size),
                        color: override_color.unwrap_or(text_color),
                        ..Default::default()
                    },
                );
                job
            });
        });
}
