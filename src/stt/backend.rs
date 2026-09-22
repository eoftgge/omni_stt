use crate::event::{PipelineError, PipelineEvent};
use async_trait::async_trait;
use std::time::Duration;

#[async_trait]
pub trait SttBackend: Send + Sync {
    async fn connect(&self) -> Result<Box<dyn SttSession>, PipelineError>;
}

#[async_trait]
pub trait SttSession: Send {
    async fn send(&mut self, audio: &[u8]) -> Result<(), PipelineError>;
    async fn recv_event(&mut self) -> Result<PipelineEvent, PipelineError>;
    async fn keepalive(&mut self) -> Result<(), PipelineError> {
        Ok(())
    }

    fn idle_timeout(&self) -> Option<Duration> {
        None
    }
}
