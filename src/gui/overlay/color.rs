use eframe::egui::Color32;
use eframe::epaint::Hsva;

pub(crate) fn get_interim_color(main_color: Color32) -> Color32 {
    let mut hsva = Hsva::from(main_color);

    if hsva.s < 0.1 {
        hsva.v = 0.65;
    } else {
        hsva.s *= 0.4;
        hsva.v = 1.0;
    }

    hsva.a = 1.0;
    Color32::from(hsva)
}
