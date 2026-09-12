use eframe::egui::Vec2;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::BoxMakeWriter;

pub mod errors;
pub mod gui;
pub mod logger;
pub mod settings;
pub mod stt;
pub mod transcription;

pub const SETTINGS_WINDOW_SIZE: Vec2 = Vec2::new(400.0, 600.0);
pub const TOOLTIP: &str = "OmniSTT";
pub const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
pub const APP_ID: &str = "omni_stt";

pub fn setup_tracing(level: Level, log_to_file: bool) -> Option<WorkerGuard> {
    let (writer, guard) = if log_to_file {
        let file_appender = tracing_appender::rolling::daily("logs", "omni.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        (BoxMakeWriter::new(non_blocking), Some(guard))
    } else {
        (BoxMakeWriter::new(std::io::stdout), None)
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(writer)
        .with_ansi(!log_to_file)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    guard
}
