use crate::stt::data::TranscriptData;
use thiserror::Error;

pub enum SttEvent {
    Connected(bool),
    Disconnected,
    Transcript(TranscriptData),
    Warning(String),
    Error(SttError),
}

#[derive(Debug, Error)]
pub enum SttError {
    #[error("Connection closed")]
    ConnectionLost,
    #[error("Disconnected, fatal error: {0}")]
    FatalAPIError(String),
    #[error("Disconnected, recoverable error: {0}")]
    RecoverableAPIError(String),
}

impl SttError {
    pub fn is_reconnect(&self) -> bool {
        matches!(self, SttError::RecoverableAPIError(_))
    }
}
