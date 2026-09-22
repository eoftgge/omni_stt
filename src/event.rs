use thiserror::Error;

pub enum PipelineEvent {
    Connected(bool),
    Disconnected,
    Transcript(TranscriptData),
    Interim(Vec<TranscriptData>),
    Warning(String),
    Error(PipelineError),
    /// The capture stream died mid-session. cpal does not restart it, so this
    /// ends the session: no audio will ever reach the worker again.
    AudioLost(String),
}

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Connection closed")]
    ConnectionLost,
    #[error("Disconnected, fatal error: {0}")]
    FatalAPIError(String),
    #[error("Disconnected, recoverable error: {0}")]
    RecoverableAPIError(String),
}

impl PipelineError {
    pub fn is_reconnect(&self) -> bool {
        matches!(
            self,
            PipelineError::RecoverableAPIError(_) | PipelineError::ConnectionLost
        )
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptData {
    pub text: String,
    pub speaker: Option<String>,
}
