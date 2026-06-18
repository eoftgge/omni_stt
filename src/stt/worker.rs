use crate::stt::action::StreamAction;
use crate::stt::event::{SttError, SttEvent};
use crate::stt::provider::SttProvider;
use crate::stt::utils::is_silent;
use crate::transcription::audio::AudioSample;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

const MAX_RETRIES: u32 = 5;
const RECONNECT_DELAY: u64 = 1000;

pub struct GenericSttWorker {
    rx_audio: Receiver<AudioSample>,
    tx_recycle: Sender<AudioSample>,
    tx_event: Sender<SttEvent>,
    hangover_chunks_limit: usize,
    vad_threshold: u32,
    provider: Box<dyn SttProvider>,
}

impl GenericSttWorker {
    pub fn new(
        rx_audio: Receiver<AudioSample>,
        tx_recycle: Sender<AudioSample>,
        tx_event: Sender<SttEvent>,
        hangover_chunks_limit: usize,
        vad_threshold: u32,
        provider: Box<dyn SttProvider>,
    ) -> Self {
        Self {
            rx_audio,
            tx_recycle,
            tx_event,
            hangover_chunks_limit,
            vad_threshold,
            provider,
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), SttError> {
        let mut retry_count = 0;
        let mut flag_first_connection = true;

        loop {
            let first_packet = if retry_count == 0 {
                let Some(packet) = self.wait_first_packet().await else {
                    return Ok(());
                };
                packet
            } else {
                Vec::new()
            };

            if self.provider.connect().await.is_err() {
                self.handle_reconnect(&mut retry_count).await?;
                continue;
            }

            retry_count = 0;

            if !first_packet.is_empty() {
                let slice = bytemuck::cast_slice(&first_packet);
                if self.provider.send(slice).await.is_err() {
                    continue;
                }
                let _ = self.tx_recycle.send(first_packet).await;
            }

            let _ = self
                .tx_event
                .send(SttEvent::Connected(flag_first_connection))
                .await;
            flag_first_connection = false;

            if self.run_session_loop().await == StreamAction::Stop {
                return Ok(());
            }
        }
    }

    async fn run_session_loop(&mut self) -> StreamAction {
        let mut hangover_counter = 0;

        loop {
            tokio::select! {
                audio_opt = self.rx_audio.recv() => {
                    let Some(mut buffer) = audio_opt else { return StreamAction::Stop; };
                    if !is_silent(&buffer, self.vad_threshold) {
                        hangover_counter = self.hangover_chunks_limit;
                    } else if hangover_counter > 0 {
                        hangover_counter = hangover_counter.saturating_sub(1);
                    } else {
                        buffer.clear();
                        let _ = self.tx_recycle.send(buffer).await;
                        continue;
                    }

                    let slice = bytemuck::cast_slice(&buffer);
                    if self.provider.send(slice).await.is_err() {
                        return StreamAction::Reconnect;
                    }

                    buffer.clear();
                    let _ = self.tx_recycle.send(buffer).await;
                }

                event_result = self.provider.recv_event() => {
                    match event_result {
                        Ok(SttEvent::Transcript(data)) => {
                            let _ = self.tx_event.send(SttEvent::Transcript(data)).await;
                        },
                        Ok(SttEvent::Disconnected) => {
                            let _ = self.tx_event.send(SttEvent::Disconnected).await;
                            return StreamAction::Reconnect;
                        },
                        Err(SttError::RecoverableAPIError(e)) => {
                            tracing::warn!("Temporary API Error: {}", e);
                            return StreamAction::Reconnect;
                        },
                        Err(e) => {
                            tracing::error!("Fatal API Error: {:?}", e);
                            let _ = self.tx_event.send(SttEvent::Error(e)).await;
                            return StreamAction::Stop;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    async fn handle_reconnect(&self, retry_count: &mut u32) -> Result<(), SttError> {
        sleep(Duration::from_millis(RECONNECT_DELAY)).await;
        *retry_count += 1;

        if *retry_count > MAX_RETRIES {
            return Err(SttError::ConnectionLost);
        }
        Ok(())
    }

    async fn wait_first_packet(&mut self) -> Option<AudioSample> {
        tracing::debug!("Waiting for speech to connect...");

        loop {
            match self.rx_audio.recv().await {
                Some(packet) if !is_silent(&packet, self.vad_threshold) => {
                    return Some(packet);
                }
                Some(mut packet) => {
                    packet.clear();
                    let _ = self.tx_recycle.send(packet).await;
                }
                None => {
                    tracing::info!("Audio channel closed by app. Stopping worker.");
                    return None;
                }
            }
        }
    }
}
