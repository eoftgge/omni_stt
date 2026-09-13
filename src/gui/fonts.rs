use eframe::egui::{Context, FontData, FontDefinitions};
use eframe::epaint::FontFamily;
use std::sync::Arc;

const FONTS: [&[u8]; 7] = [
    include_bytes!("../../assets/fonts/NotoSans-Medium.ttf"),
    include_bytes!("../../assets/fonts/NotoSansSC-Medium.ttf"),
    include_bytes!("../../assets/fonts/NotoSansJP-Medium.ttf"),
    include_bytes!("../../assets/fonts/NotoSansArabic-Medium.ttf"),
    include_bytes!("../../assets/fonts/NotoSansKR-Medium.ttf"),
    include_bytes!("../../assets/fonts/NotoSansTC-Medium.ttf"),
    include_bytes!("../../assets/fonts/NotoSansGunjalaGondi-Medium.ttf"),
];

pub fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    for (n, font) in FONTS.iter().enumerate() {
        fonts
            .font_data
            .insert(n.to_string(), Arc::new(FontData::from_static(font)));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(n, n.to_string());
    }

    ctx.set_fonts(fonts);
}
