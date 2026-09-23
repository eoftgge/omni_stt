use crate::settings::anchor::Anchor;
use eframe::egui::{Align2, Color32, Vec2, vec2};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsUI {
    pub(crate) max_blocks: usize,
    pub(crate) offset: (f32, f32),
    pub(crate) anchor: Anchor,
    pub(crate) font_size: usize,
    pub(crate) background_color: [u8; 4],
    pub(crate) text_color: [u8; 3],
    pub(crate) enable_high_priority: bool,
    pub(crate) is_text_outline: bool,
    pub(crate) max_lines: usize,
    pub(crate) monitor: Option<String>,
}

impl SettingsUI {
    pub const DEFAULT_BACKGROUND_COLOR: [u8; 4] = [0, 0, 0, 150];
    pub const DEFAULT_TEXT_COLOR: [u8; 3] = [255, 255, 0]; // yellow
}

impl Default for SettingsUI {
    fn default() -> Self {
        Self {
            enable_high_priority: true,
            offset: (0.0, -30.0),
            anchor: Anchor::CenterBottom,
            font_size: 21,
            background_color: Self::DEFAULT_BACKGROUND_COLOR,
            text_color: Self::DEFAULT_TEXT_COLOR,
            max_blocks: 3,
            is_text_outline: false,
            max_lines: 3,
            monitor: None,
        }
    }
}

impl SettingsUI {
    pub fn text_color(&self) -> Color32 {
        let color = self.text_color;
        Color32::from_rgb(color[0], color[1], color[2])
    }

    pub fn background_color(&self) -> Color32 {
        let color = self.background_color;
        Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
    }

    pub fn get_anchor(&self) -> (Align2, Vec2) {
        (self.anchor.align(), vec2(self.offset.0, self.offset.1))
    }
}
