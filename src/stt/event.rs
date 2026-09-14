use crate::stt::data::TranscriptData;
use thiserror::Error;

pub enum SttEvent {
    Connected(bool),
    Disconnected,
    Transcript(TranscriptData),
    Interim(Vec<TranscriptData>),
    Warning(String),
    Error(SttError),
    /// The capture stream died mid-session. cpal does not restart it, so this
    /// ends the session: no audio will ever reach the worker again.
    AudioLost(String),
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
        matches!(
            self,
            SttError::RecoverableAPIError(_) | SttError::ConnectionLost
        )
    }
}
