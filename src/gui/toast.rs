use eframe::egui::WidgetText;
use egui_toast::{Toast, ToastKind, ToastOptions, ToastStyle, Toasts};

const SUCCESS_SECONDS: f64 = 3.0;
const INFO_SECONDS: f64 = 4.0;
const WARN_SECONDS: f64 = 5.0;
const ERROR_SECONDS: f64 = 8.0;

pub trait Notify {
    fn notify(&mut self, kind: ToastKind, seconds: f64, text: impl Into<WidgetText>);

    fn success(&mut self, text: impl Into<WidgetText>) {
        self.notify(ToastKind::Success, SUCCESS_SECONDS, text);
    }

    fn info(&mut self, text: impl Into<WidgetText>) {
        self.notify(ToastKind::Info, INFO_SECONDS, text);
    }

    fn warn(&mut self, text: impl Into<WidgetText>) {
        self.notify(ToastKind::Warning, WARN_SECONDS, text);
    }

    fn error(&mut self, text: impl Into<WidgetText>) {
        self.notify(ToastKind::Error, ERROR_SECONDS, text);
    }
}

impl Notify for Toasts {
    /// escape hatch for the rare toast that needs its own timing
    fn notify(&mut self, kind: ToastKind, seconds: f64, text: impl Into<WidgetText>) {
        self.add(Toast {
            text: text.into(),
            kind,
            style: ToastStyle::default(),
            options: ToastOptions::default().duration_in_seconds(seconds),
        });
    }
}
