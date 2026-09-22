pub mod ffi;
pub mod model;
pub mod probe;
pub mod types;

use crate::event::TranscriptData;
use crate::event::{PipelineError, PipelineEvent};
use crate::stt::adapters::vosk::ffi::VoskApi;
use crate::stt::adapters::vosk::model::{Decoding, Model, Recognizer};
use crate::stt::adapters::vosk::types::{VoskPartial, VoskText};
use crate::stt::backend::{SttBackend, SttSession};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};

fn parse<T: serde::de::DeserializeOwned>(json: &str) -> Option<T> {
    match serde_json::from_str(json) {
        Ok(value) => Some(value),
        Err(e) => {
            tracing::warn!("Vosk returned a non-JSON payload: {e}");
            None
        }
    }
}

fn run_recognition_loop(
    model: Arc<Model>,
    mut audio_rx: Receiver<Vec<i16>>,
    event_tx: Sender<PipelineEvent>,
) {
    let mut recognizer = match Recognizer::new(model, 16000.0) {
        Ok(r) => r,
        Err(e) => {
            let _ = event_tx.blocking_send(PipelineEvent::Error(PipelineError::FatalAPIError(e)));
            return;
        }
    };

    while let Some(chunk) = audio_rx.blocking_recv() {
        let Some(event) = process_chunk(&mut recognizer, &chunk) else {
            continue;
        };
        if event_tx.blocking_send(event).is_err() {
            break;
        }
    }

    if let Some(parsed) = parse::<VoskText>(&recognizer.final_result()) {
        let text = parsed.text.trim();
        if !text.is_empty() {
            let _ = event_tx.blocking_send(PipelineEvent::Transcript(TranscriptData {
                text: format!("{text} "),
                speaker: None,
            }));
        }
    }
}

fn process_chunk(recognizer: &mut Recognizer, chunk: &[i16]) -> Option<PipelineEvent> {
    match recognizer.accept(chunk) {
        Decoding::Final => {
            let parsed: VoskText = parse(&recognizer.result())?;
            let text = parsed.text.trim();
            if text.is_empty() {
                return None;
            }
            Some(PipelineEvent::Transcript(TranscriptData {
                text: format!("{text} "),
                speaker: None,
            }))
        }
        Decoding::Partial => {
            let parsed: VoskPartial = parse(&recognizer.partial_result())?;
            let text = parsed.partial.trim();
            if text.is_empty() {
                return None;
            }
            Some(PipelineEvent::Interim(vec![TranscriptData {
                text: text.into(),
                speaker: None,
            }]))
        }
        Decoding::Failed => {
            tracing::warn!("Vosk decoding failed for this chunk");
            None
        }
    }
}

pub struct VoskBackend {
    model: Arc<Model>,
}

impl VoskBackend {
    pub async fn new(
        model_path: impl Into<PathBuf>,
        library_path: Option<PathBuf>,
    ) -> Result<Self, PipelineError> {
        let model_path = model_path.into();

        let model = tokio::task::spawn_blocking(move || {
            let api = VoskApi::load(library_path.as_deref())?;
            Model::load(Arc::new(api), &model_path)
        })
        .await
        .map_err(|_| PipelineError::FatalAPIError("Vosk load task panicked".into()))?
        .map_err(PipelineError::FatalAPIError)?;

        Ok(Self {
            model: Arc::new(model),
        })
    }
}

#[async_trait]
impl SttBackend for VoskBackend {
    async fn connect(&self) -> Result<Box<dyn SttSession>, PipelineError> {
        let model = Arc::clone(&self.model);

        let (audio_tx, audio_rx) = channel::<Vec<i16>>(100);
        let (event_tx, event_rx) = channel::<PipelineEvent>(100);

        tokio::task::spawn_blocking(move || run_recognition_loop(model, audio_rx, event_tx));
        Ok(Box::new(VoskSession { audio_tx, event_rx }))
    }
}

pub struct VoskSession {
    pub(super) audio_tx: Sender<Vec<i16>>,
    pub(super) event_rx: Receiver<PipelineEvent>,
}

#[async_trait]
impl SttSession for VoskSession {
    async fn send(&mut self, audio: &[u8]) -> Result<(), PipelineError> {
        let audio_i16: &[i16] = bytemuck::cast_slice(audio);
        self.audio_tx
            .send(audio_i16.to_vec())
            .await
            .map_err(|_| PipelineError::FatalAPIError("Vosk audio channel closed".into()))
    }

    async fn recv_event(&mut self) -> Result<PipelineEvent, PipelineError> {
        self.event_rx
            .recv()
            .await
            .ok_or_else(|| PipelineError::FatalAPIError("Vosk event channel closed".into()))
    }
}
