pub mod connection;
pub mod request;
pub mod session;
pub mod types;

use async_trait::async_trait;
use std::collections::VecDeque;
use tungstenite::{Bytes, Message};

use crate::errors::OmniSttErrors;
use crate::stt::adapters::soniox::types::SonioxTranscriptionToken;
use crate::stt::prelude::{SttBackend, SttError, SttEvent, SttSession, TranscriptData};
use connection::SonioxConnection;
use session::{SonioxSessionReader, SonioxSessionWriter};
use types::{SonioxTranscriptionMessage, SonioxTranscriptionRequest};

const ERROR_CODES_RECONNECT: &[usize] = &[408, 502, 503];
const URL: &str = "wss://stt-rt.soniox.com/transcribe-websocket";
const MODEL: &str = "stt-rt-v4";
const READ_TIMEOUT: std::time::Duration = std::time::Duration::from_mins(1);

fn classify_connect_error(err: OmniSttErrors) -> SttError {
    match err {
        OmniSttErrors::WebSocket(tungstenite::Error::Http(resp))
            if matches!(resp.status().as_u16(), 400 | 401 | 403) =>
        {
            SttError::FatalAPIError(format!("Handshake rejected: {}", resp.status()))
        }
        other => SttError::RecoverableAPIError(other.to_string()),
    }
}

pub struct SonioxBackend {
    request: SonioxTranscriptionRequest,
}

pub struct SonioxSession {
    pub(super) writer: SonioxSessionWriter,
    pub(super) reader: SonioxSessionReader,
    pub(super) event_queue: VecDeque<SttEvent>,
    pub(super) deadline: tokio::time::Instant,
}

impl SonioxBackend {
    pub fn new(request: SonioxTranscriptionRequest) -> Self {
        Self { request }
    }
}

#[async_trait]
impl SttBackend for SonioxBackend {
    async fn connect(&self) -> Result<Box<dyn SttSession>, SttError> {
        let conn = SonioxConnection::connect(URL)
            .await
            .map_err(classify_connect_error)?;
        let (writer, reader) = conn
            .into_session(&self.request)
            .await
            .map_err(classify_connect_error)?;

        Ok(Box::new(SonioxSession {
            writer,
            reader,
            event_queue: VecDeque::new(),
            deadline: tokio::time::Instant::now() + READ_TIMEOUT,
        }))
    }
}

impl SonioxSession {
    async fn handle_ws_message(&mut self, msg: Message) -> Result<Option<SttEvent>, SttError> {
        match msg {
            Message::Text(txt) => self.handle_text_message(&txt),
            Message::Ping(data) => {
                let _ = self.writer.send_pong(data).await;
                Ok(None)
            }
            Message::Pong(_) => {
                tracing::debug!("Pong received");
                Ok(None)
            }
            Message::Close(_) => {
                tracing::warn!("Server sent Close frame");
                Ok(Some(SttEvent::Disconnected))
            }
            _ => Ok(None),
        }
    }

    fn handle_text_message(&mut self, txt: &str) -> Result<Option<SttEvent>, SttError> {
        tracing::debug!("soniox raw: {txt}");
        let parsed_msg: SonioxTranscriptionMessage = serde_json::from_str(txt)
            .map_err(|e| SttError::FatalAPIError(format!("JSON parse error: {}", e)))?;

        match parsed_msg {
            SonioxTranscriptionMessage::Response(r) => {
                self.enqueue_tokens(r.tokens);
                Ok(self.event_queue.pop_front())
            }
            SonioxTranscriptionMessage::Error(e) => {
                if ERROR_CODES_RECONNECT.contains(&e.error_code) {
                    Err(SttError::RecoverableAPIError(e.error_message))
                } else {
                    Err(SttError::FatalAPIError(e.error_message))
                }
            }
        }
    }

    fn enqueue_tokens(&mut self, tokens: Vec<SonioxTranscriptionToken>) {
        let had_tokens = !tokens.is_empty();
        let mut final_text = String::new();
        let mut interim_text = String::new();
        let mut interims: Vec<TranscriptData> = Vec::new();
        let mut current_speaker = None;

        for token in tokens {
            if token.translation_status.as_deref() == Some("original") {
                continue;
            }

            let token_speaker = token.speaker.clone();

            if current_speaker.is_some() && current_speaker != token_speaker {
                self.flush_buffers(
                    &mut final_text,
                    &mut interim_text,
                    &current_speaker,
                    &mut interims,
                );
            }

            current_speaker = token_speaker;
            if token.is_final {
                final_text.push_str(&token.text);
            } else {
                interim_text.push_str(&token.text);
            }
        }

        self.flush_buffers(
            &mut final_text,
            &mut interim_text,
            &current_speaker,
            &mut interims,
        );

        if had_tokens {
            self.event_queue.push_back(SttEvent::Interim(interims));
        }
    }

    fn flush_buffers(
        &mut self,
        final_text: &mut String,
        interim_text: &mut String,
        speaker: &Option<String>,
        interims: &mut Vec<TranscriptData>,
    ) {
        if !final_text.is_empty() {
            self.event_queue
                .push_back(SttEvent::Transcript(TranscriptData {
                    text: std::mem::take(final_text),
                    speaker: speaker.clone(),
                }));
        }
        if !interim_text.is_empty() {
            interims.push(TranscriptData {
                text: std::mem::take(interim_text),
                speaker: speaker.clone(),
            });
        }
    }
}

#[async_trait]
impl SttSession for SonioxSession {
    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError> {
        self.writer
            .send_bytes(Bytes::copy_from_slice(audio))
            .await
            .map_err(|_| SttError::ConnectionLost)
    }

    async fn recv_event(&mut self) -> Result<SttEvent, SttError> {
        if let Some(event) = self.event_queue.pop_front() {
            return Ok(event);
        }

        loop {
            let msg = tokio::select! {
                res = self.reader.recv_message() => res.map_err(|e| {
                    tracing::error!("WS Error/EOF: {e}");
                    SttError::ConnectionLost
                })?,
                _ = tokio::time::sleep_until(self.deadline) => {
                    tracing::warn!("No frames from Soniox for {READ_TIMEOUT:?}");
                    return Err(SttError::ConnectionLost);
                }
            };

            self.deadline = tokio::time::Instant::now() + READ_TIMEOUT;
            if let Some(event) = self.handle_ws_message(msg).await? {
                return Ok(event);
            }
        }
    }

    async fn keepalive(&mut self) -> Result<(), SttError> {
        self.writer
            .send_ping()
            .await
            .map_err(|_| SttError::ConnectionLost)
    }
}
