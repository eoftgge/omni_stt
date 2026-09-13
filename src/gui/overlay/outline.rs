use eframe::egui::{Color32, Vec2};

#[derive(Clone, Copy)]
pub struct TextOutline {
    pub color: Color32,
    pub width: f32,
}

impl TextOutline {
    pub(crate) fn offsets(&self) -> impl Iterator<Item = Vec2> + '_ {
        const DIRS: [(f32, f32); 8] = [
            (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0),
            (-1.0,  0.0),              (1.0,  0.0),
            (-1.0,  1.0), (0.0,  1.0), (1.0,  1.0),
        ];
        DIRS.iter().map(move |&(x, y)| Vec2::new(x, y) * self.width)
    }
}