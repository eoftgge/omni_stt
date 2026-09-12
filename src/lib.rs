pub mod errors;
pub mod gui;
pub mod logger;
pub mod settings;
pub mod stt;
pub mod transcription;

pub const SETTINGS_WINDOW_SIZE: eframe::egui::Vec2 = eframe::egui::Vec2::new(400.0, 600.0);
pub const TOOLTIP: &str = "OmniSTT";
pub const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");
pub const APP_ID: &str = "omni_stt";
pub const CONFIG_PATH: &str = "omni.toml";
