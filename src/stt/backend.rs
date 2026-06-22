use crate::stt::event::{SttError, SttEvent};
use async_trait::async_trait;

#[async_trait]
pub trait SttBackend: Send + Sync {
    async fn connect(&self) -> Result<Box<dyn SttSession>, SttError>;
}

#[async_trait]
pub trait SttSession: Send {
    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError>;
    async fn recv_event(&mut self) -> Result<SttEvent, SttError>;
}
