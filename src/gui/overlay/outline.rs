use eframe::egui::{Color32, Vec2};

#[derive(Clone, Copy)]
pub struct TextOutline {
    pub color: Color32,
    pub width: f32,
}

impl TextOutline {
    pub(crate) fn offsets(&self) -> impl Iterator<Item = Vec2> + '_ {
        const DIRS: [(f32, f32); 8] = [
            (-1.0, -1.0),
            (0.0, -1.0),
            (1.0, -1.0),
            (-1.0, 0.0),
            (1.0, 0.0),
            (-1.0, 1.0),
            (0.0, 1.0),
            (1.0, 1.0),
        ];
        DIRS.iter().map(move |&(x, y)| Vec2::new(x, y) * self.width)
    }

    pub fn for_font_size(font_size: f32) -> Self {
        Self {
            color: Color32::from_black_alpha(220),
            width: (font_size / 14.0).clamp(1.0, 3.0),
        }
    }
}
