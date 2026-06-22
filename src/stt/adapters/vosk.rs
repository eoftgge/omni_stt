use crate::stt::data::TranscriptData;
use crate::stt::event::{SttError, SttEvent};
use crate::stt::backend::{SttBackend, SttSession};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use vosk::{DecodingState, Model, Recognizer};

fn run_recognition_loop(
    model: Arc<Model>,
    mut audio_rx: Receiver<Vec<i16>>,
    event_tx: Sender<SttEvent>,
) {
    let mut recognizer = match Recognizer::new(&model, 16000.0) {
        Some(r) => r,
        None => {
            let _ = event_tx.blocking_send(SttEvent::Error(SttError::FatalAPIError(
                "Failed to create Vosk recognizer".into(),
            )));
            return;
        }
    };

    while let Some(chunk) = audio_rx.blocking_recv() {
        if let Some(event) = process_chunk(&mut recognizer, &chunk) {
            if event_tx.blocking_send(event).is_err() {
                break;
            }
        }
    }
}

fn process_chunk(recognizer: &mut Recognizer, chunk: &[i16]) -> Option<SttEvent> {
    match recognizer.accept_waveform(chunk) {
        Ok(DecodingState::Finalized) => {
            let text = recognizer.result().single()?.text.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(SttEvent::Transcript(TranscriptData {
                text: format!("{text} "),
                is_final: true,
                speaker: None,
            }))
        }
        Ok(DecodingState::Running) => {
            let text = recognizer.partial_result().partial.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(SttEvent::Transcript(TranscriptData {
                text,
                is_final: false,
                speaker: None,
            }))
        }
        Ok(DecodingState::Failed) => {
            tracing::warn!("Vosk decoding failed for this chunk");
            None
        }
        Err(e) => {
            tracing::error!("Error passing audio to Vosk: {:?}", e);
            None
        }
    }
}

pub struct VoskBackend {
    model: Arc<Model>,
}

impl VoskBackend {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, SttError> {
        let path_str = path.into().to_string_lossy().to_string();
        let model = Model::new(&path_str)
            .ok_or_else(|| SttError::FatalAPIError("Failed to load Vosk model".into()))?;
        Ok(Self {
            model: Arc::new(model),
        })
    }
}

#[async_trait]
impl SttBackend for VoskBackend {
    async fn connect(&self) -> Result<Box<dyn SttSession>, SttError> {
        let model = Arc::clone(&self.model);

        let (audio_tx, audio_rx) = channel::<Vec<i16>>(100);
        let (event_tx, event_rx) = channel::<SttEvent>(100);

        tokio::task::spawn_blocking(move || run_recognition_loop(model, audio_rx, event_tx));
        Ok(Box::new(VoskSession {
            audio_tx,
            event_rx,
        }))
    }
}

pub struct VoskSession {
    pub(super) audio_tx: Sender<Vec<i16>>,
    pub(super) event_rx: Receiver<SttEvent>,
}

#[async_trait]
impl SttSession for VoskSession {
    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError> {
        let audio_i16: &[i16] = bytemuck::cast_slice(audio);
        self.audio_tx
            .send(audio_i16.to_vec())
            .await
            .map_err(|_| SttError::FatalAPIError("Vosk audio channel closed".into()))
    }

    async fn recv_event(&mut self) -> Result<SttEvent, SttError> {
        self.event_rx
            .recv()
            .await
            .ok_or_else(|| SttError::FatalAPIError("Vosk event channel closed".into()))
    }
}
