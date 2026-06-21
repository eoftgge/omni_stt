use crate::stt::data::TranscriptData;
use crate::stt::event::{SttError, SttEvent};
use crate::stt::provider::SttProvider;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use vosk::{DecodingState, Model, Recognizer};

pub struct VoskAdapter {
    path: PathBuf,
    audio_tx: Option<Sender<Vec<i16>>>,
    event_rx: Option<Receiver<SttEvent>>,
}

impl VoskAdapter {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            audio_tx: None,
            event_rx: None,
        }
    }
}

#[async_trait]
impl SttProvider for VoskAdapter {
    async fn connect(&mut self) -> Result<(), SttError> {
        let path_str = self.path.to_string_lossy().to_string();

        let (audio_tx, mut audio_rx) = channel::<Vec<i16>>(100);
        let (event_tx, event_rx) = channel::<SttEvent>(100);

        self.audio_tx = Some(audio_tx);
        self.event_rx = Some(event_rx);

        let event_tx_clone = event_tx.clone();

        tokio::task::spawn_blocking(move || {
            let model = match Model::new(&path_str) {
                Some(m) => m,
                None => {
                    let _ = event_tx_clone
                        .blocking_send(SttEvent::Error(SttError::FatalAPIError("Failed to load Vosk model".into())));
                    return;
                }
            };

            let mut recognizer =
                Recognizer::new(&model, 16000.0).expect("Failed to create Vosk recognizer");

            let _ = event_tx_clone.blocking_send(SttEvent::Connected(true));

            while let Some(chunk) = audio_rx.blocking_recv() {
                match recognizer.accept_waveform(&chunk) {
                    Ok(DecodingState::Finalized) => {
                        let result = recognizer.result();
                        if let Some(single) = result.single() {
                            let text = single.text.trim().to_string();

                            if !text.is_empty() {
                                let data = TranscriptData {
                                    text,
                                    is_final: true,
                                    speaker: None,
                                };
                                let _ = event_tx_clone.blocking_send(SttEvent::Transcript(data));
                            }
                        }
                    }
                    Ok(DecodingState::Running) => {
                        let partial = recognizer.partial_result();
                        let text = partial.partial.trim().to_string();

                        if !text.is_empty() {
                            let data = TranscriptData {
                                text,
                                is_final: false,
                                speaker: None,
                            };
                            let _ = event_tx_clone.blocking_send(SttEvent::Transcript(data));
                        }
                    }
                    Ok(DecodingState::Failed) => eprintln!("Vosk decoding failed for this chunk"),
                    Err(e) => eprintln!("Error passing audio to Vosk: {:?}", e),
                }
            }
        });

        Ok(())
    }

    async fn send(&mut self, audio: &[u8]) -> Result<(), SttError> {
        if let Some(tx) = &self.audio_tx {
            let audio_i16: &[i16] = bytemuck::cast_slice(audio);
            tx.send(audio_i16.to_vec())
                .await
                .map_err(|_| SttError::FatalAPIError("Vosk audio channel closed".into()))?;
        }
        Ok(())
    }

    async fn recv_event(&mut self) -> Result<SttEvent, SttError> {
        if let Some(rx) = &mut self.event_rx {
            rx.recv()
                .await
                .ok_or_else(|| SttError::FatalAPIError("Vosk event channel closed".into()))
        } else {
            Err(SttError::FatalAPIError(
                "Vosk provider not connected".into(),
            ))
        }
    }
}
