use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, Response, Sense, Ui, Vec2};

pub fn add_outlined_text(
    ui: &mut Ui,
    outline: Option<TextOutline>,
    fallback_color: Color32,
    make_job: impl Fn(Option<Color32>) -> LayoutJob,
) -> Response {
    let pad = outline.map_or(0.0, |o| o.width);

    let main = ui.fonts_mut(|f| f.layout_job(make_job(None)));
    let (rect, response) =
        ui.allocate_exact_size(main.size() + Vec2::splat(pad * 2.0), Sense::hover());
    let pos = rect.min + Vec2::splat(pad);

    if let Some(outline) = outline {
        let shadow = ui.fonts_mut(|f| f.layout_job(make_job(Some(outline.color))));
        for offset in outline.offsets() {
            ui.painter()
                .galley(pos + offset, shadow.clone(), outline.color);
        }
    }

    ui.painter().galley(pos, main, fallback_color);
    response
}

#[derive(Clone, Copy)]
pub struct TextOutline {
    pub color: Color32,
    pub width: f32,
}

impl TextOutline {
    pub fn for_font_size(font_size: f32) -> Self {
        Self {
            color: Color32::from_black_alpha(220),
            width: (font_size / 14.0).clamp(1.0, 3.0),
        }
    }

    #[rustfmt::skip]
    pub(crate) fn offsets(&self) -> impl Iterator<Item = Vec2> + '_ {
        const DIRS: [(f32, f32); 8] = [
            (-1.0, -1.0), (0.0, -1.0), (1.0, -1.0),
            (-1.0,  0.0),              (1.0,  0.0),
            (-1.0,  1.0), (0.0,  1.0), (1.0,  1.0),
        ];
        DIRS.iter().map(move |&(x, y)| Vec2::new(x, y) * self.width)
    }
}
