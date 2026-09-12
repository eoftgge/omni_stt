use crate::errors::OmniSttErrors;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::{Bytes, Message, Utf8Bytes};

pub(crate) type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub struct SonioxSessionReader(pub(super) SplitStream<WsStream>);
pub struct SonioxSessionWriter(pub(super) SplitSink<WsStream, Message>);

impl SonioxSessionReader {
    pub async fn recv_message(&mut self) -> Result<Message, OmniSttErrors> {
        match self.0.next().await {
            Some(Ok(msg)) => Ok(msg),
            Some(Err(e)) => Err(e.into()),
            None => Err(OmniSttErrors::ConnectionLost),
        }
    }
}

impl SonioxSessionWriter {
    pub async fn send_pong(&mut self, data: Bytes) -> Result<(), OmniSttErrors> {
        tracing::debug!("Sending pong");
        self.0.send(Message::Pong(data)).await?;
        Ok(())
    }

    pub async fn send_text(&mut self, data: impl Into<Utf8Bytes>) -> Result<(), OmniSttErrors> {
        let message = Message::text(data.into());
        self.0.send(message).await?;
        Ok(())
    }

    pub async fn send_bytes(&mut self, data: impl Into<Bytes>) -> Result<(), OmniSttErrors> {
        let message = Message::Binary(data.into());
        self.0.send(message).await?;
        Ok(())
    }

    pub async fn send_ping(&mut self) -> Result<(), OmniSttErrors> {
        tracing::debug!("Sending ping");
        self.0.send(Message::Ping(Bytes::new())).await?;
        Ok(())
    }
}
