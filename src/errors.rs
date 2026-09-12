use crate::stt::event::SttError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OmniSttErrors {
    #[error("Audio error: {0}")]
    Audio(#[from] cpal::Error),
    #[error("WebSocket connection error: {0}")]
    WebSocket(#[from] tungstenite::Error),
    #[error("Server connection lost (Heartbeat failed)")]
    ConnectionLost,
    #[error("Output device is not found")]
    NotFoundOutputDevice,
    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Failed to serialize configuration: {0}")]
    ConfigSave(#[from] toml::ser::Error),
    #[error("Failed to load configuration: {0}")]
    ConfigLoad(#[from] toml::de::Error),
    #[error("Invalid UTF-8 sequence: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Filesystem I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Internal application error: {0}")]
    Internal(String),
    #[error("API error {0}: {1}\nStopping audio...")]
    API(usize, String),
    #[error("Provider error: {0}\nStopping audio...")]
    App(#[from] SttError),
    #[error("Tray icon error: {0}")]
    Tray(#[from] tray_icon::Error),
    #[error("Invalid tray icon image: {0}")]
    TrayIcon(#[from] tray_icon::BadIcon),
    #[error("Invalid tray icon menu: {0}")]
    TrayMenu(#[from] tray_icon::menu::Error),
    #[error("Key storage error: {0}")]
    KeyStore(#[from] keyring::Error),
}

impl From<&str> for OmniSttErrors {
    fn from(s: &str) -> Self {
        OmniSttErrors::Internal(s.to_string())
    }
}
