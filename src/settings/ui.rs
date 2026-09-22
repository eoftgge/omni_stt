use eframe::egui::{Align2, Color32, Vec2, vec2};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsUI {
    pub(crate) max_blocks: usize,
    pub(crate) offset: (f32, f32),
    pub(crate) anchor: usize,
    pub(crate) font_size: usize,
    pub(crate) background_color: [u8; 4],
    pub(crate) text_color: [u8; 3],
    pub(crate) enable_high_priority: bool,
    pub(crate) is_text_outline: bool,
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
            anchor: 7,
            font_size: 21,
            background_color: Self::DEFAULT_BACKGROUND_COLOR,
            text_color: Self::DEFAULT_TEXT_COLOR,
            max_blocks: 3,
            is_text_outline: false,
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
        let align = match self.anchor {
            0 => Align2::LEFT_TOP,
            1 => Align2::CENTER_TOP,
            2 => Align2::RIGHT_TOP,
            3 => Align2::LEFT_CENTER,
            4 => Align2::CENTER_CENTER,
            5 => Align2::RIGHT_CENTER,
            6 => Align2::LEFT_BOTTOM,
            7 => Align2::CENTER_BOTTOM,
            8 => Align2::RIGHT_BOTTOM,
            _ => Align2::CENTER_BOTTOM,
        };
        (align, vec2(self.offset.0, self.offset.1))
    }
}