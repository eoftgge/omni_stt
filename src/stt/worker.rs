#[cfg(test)]
mod tests;

use crate::audio::utils::{is_silent, rms};
use crate::audio::{AudioSample, CHUNK_PERIOD};
use crate::event::{PipelineError, PipelineEvent};
use crate::stt::action::StreamAction;
use crate::stt::backend::{SttBackend, SttSession};
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::timeout;

const PING_INTERVAL: Duration = Duration::from_secs(10);

/// With the network gone, one connect attempt can hang for as long as the OS
/// waits on TCP, about 21 s on Windows. Cut it short and count it as a failure.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Retry delays double from one second up to this, so a network that comes
/// back is picked up within seconds even late in an outage.
const MAX_BACKOFF: Duration = Duration::from_secs(15);

/// About ten minutes of trying with the delays above: long enough to ride out
/// Wi-Fi dropping in a lecture hall, short of retrying forever on a laptop
/// that has left the network for good.
const MAX_RETRIES: u32 = 40;

/// How far back audio queued during a reconnect is still sent: enough to catch
/// the start of the sentence being spoken, without subtitling speech from the
/// middle of the outage.
const RECONNECT_TAIL: Duration = Duration::from_secs(2);
const RECONNECT_TAIL_CHUNKS: usize =
    (RECONNECT_TAIL.as_millis() / CHUNK_PERIOD.as_millis()) as usize;

pub struct GenericSttWorker {
    rx_audio: Receiver<AudioSample>,
    tx_recycle: Sender<AudioSample>,
    tx_event: Sender<PipelineEvent>,
    hangover_chunks_limit: usize,
    vad_threshold: u32,
    backend: Box<dyn SttBackend>,
}

impl GenericSttWorker {
    pub fn new(
        rx_audio: Receiver<AudioSample>,
        tx_recycle: Sender<AudioSample>,
        tx_event: Sender<PipelineEvent>,
        hangover_chunks_limit: usize,
        vad_threshold: u32,
        backend: Box<dyn SttBackend>,
    ) -> Self {
        Self {
            rx_audio,
            tx_recycle,
            tx_event,
            hangover_chunks_limit,
            vad_threshold,
            backend,
        }
    }

    pub(crate) async fn run(mut self) -> Result<(), PipelineError> {
        let mut retry_count = 0;
        let mut flag_first_connection = true;

        loop {
            // Chunks that go out ahead of the live stream: the one that woke
            // us up, or on a reconnect the newest of what queued meanwhile.
            let mut backlog = VecDeque::new();
            if retry_count == 0 {
                let Some(packet) = self.wait_first_packet().await else {
                    return Ok(());
                };
                backlog.push_back(packet);
            }

            let mut session = match self.connect().await {
                Ok(s) => s,
                Err(e) if e.is_reconnect() => {
                    tracing::warn!("Connect failed: {e}");
                    self.handle_reconnect(&mut retry_count).await?;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Fatal connect error: {:?}", e);
                    let _ = self.tx_event.send(PipelineEvent::Error(e)).await;
                    return Ok(());
                }
            };

            if retry_count > 0 {
                backlog = self.take_recent_audio().await;
            }
            if !self.send_backlog(&mut session, backlog).await {
                self.handle_reconnect(&mut retry_count).await?;
                continue;
            }

            let _ = self
                .tx_event
                .send(PipelineEvent::Connected(flag_first_connection))
                .await;
            flag_first_connection = false;

            match self.run_session_loop(&mut session).await {
                StreamAction::Stop => return Ok(()),
                StreamAction::Idle => {
                    retry_count = 0;
                    continue;
                }
                StreamAction::Reconnect { transcribed } => {
                    if transcribed {
                        retry_count = 0;
                    }
                    self.handle_reconnect(&mut retry_count).await?
                }
            }
        }
    }

    async fn run_session_loop(&mut self, session: &mut Box<dyn SttSession>) -> StreamAction {
        let mut hangover_counter = 0;
        let mut transcribed = false;
        let mut next_ping = tokio::time::Instant::now() + PING_INTERVAL;
        let idle_timeout = session.idle_timeout();
        let mut idle_deadline = tokio::time::Instant::now() + idle_timeout.unwrap_or(PING_INTERVAL);

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
                    let res = session.send(slice).await;

                    buffer.clear();
                    let _ = self.tx_recycle.send(buffer).await;

                    if res.is_err() {
                        return StreamAction::Reconnect { transcribed };
                    }

                    if let Some(timeout) = idle_timeout {
                        idle_deadline = tokio::time::Instant::now() + timeout;
                    }
                }

                event_result = session.recv_event() => {
                    match event_result {
                        Ok(PipelineEvent::Transcript(data)) => {
                            transcribed = true;
                            let _ = self.tx_event.send(PipelineEvent::Transcript(data)).await;
                        },
                        Ok(PipelineEvent::Interim(segments)) => {
                            if !segments.is_empty() { transcribed = true; }
                            let _ = self.tx_event.send(PipelineEvent::Interim(segments)).await;
                        },
                        Ok(PipelineEvent::Warning(msg)) => {
                            tracing::warn!("Provider warning: {}", msg);
                            let _ = self.tx_event.send(PipelineEvent::Warning(msg)).await;
                        },
                        Ok(PipelineEvent::Error(e)) => {
                            tracing::error!("Provider error event: {:?}", e);
                            let _ = self.tx_event.send(PipelineEvent::Error(e)).await;
                            return StreamAction::Stop;
                        }
                        Ok(PipelineEvent::Connected(_)) => {},
                        Ok(PipelineEvent::Disconnected) => {
                            let _ = self.tx_event.send(PipelineEvent::Disconnected).await;
                            return StreamAction::Reconnect { transcribed } ;
                        },
                        // a backend never emits this: AudioSession reports
                        // device loss straight to the GUI, bypassing the worker
                        Ok(PipelineEvent::AudioLost(_)) => {}
                        Err(e) if e.is_reconnect() => {
                            tracing::warn!("Recoverable error: {}", e);
                            return StreamAction::Reconnect { transcribed } ;
                        }
                        Err(e) => {
                            tracing::error!("Fatal API Error: {:?}", e);
                            let _ = self.tx_event.send(PipelineEvent::Error(e)).await;
                            return StreamAction::Stop;
                        }
                    }
                },
                _ = tokio::time::sleep_until(next_ping) => {
                    if session.keepalive().await.is_err() {
                        return StreamAction::Reconnect { transcribed };
                    }
                    next_ping = tokio::time::Instant::now() + PING_INTERVAL;
                },
                _ = tokio::time::sleep_until(idle_deadline), if idle_timeout.is_some() => {
                    tracing::debug!("No audio for {idle_timeout:?}, closing the session before the server does");
                    return StreamAction::Idle;
                }
            }
        }
    }

    /// `backend.connect()`, given up on after `CONNECT_TIMEOUT`. A timeout is
    /// a network failure like any other and gets retried.
    async fn connect(&self) -> Result<Box<dyn SttSession>, PipelineError> {
        timeout(CONNECT_TIMEOUT, self.backend.connect())
            .await
            .unwrap_or_else(|_| {
                Err(PipelineError::RecoverableAPIError(format!(
                    "no answer within {CONNECT_TIMEOUT:?}"
                )))
            })
    }

    async fn handle_reconnect(&mut self, retry_count: &mut u32) -> Result<(), PipelineError> {
        *retry_count += 1;
        if *retry_count > MAX_RETRIES {
            return Err(PipelineError::ConnectionLost);
        }

        let delay = backoff(*retry_count);
        tracing::info!("Reconnecting in {delay:?}, attempt {retry_count} of {MAX_RETRIES}");
        self.discard_audio_for(delay).await;
        Ok(())
    }

    /// Sleeps for `delay`, throwing away whatever the mixer sends meanwhile.
    /// It keeps ticking without a session, and left alone its backlog grows to
    /// minutes of speech, all of it subtitled late once the connection is back.
    async fn discard_audio_for(&mut self, delay: Duration) {
        let deadline = tokio::time::Instant::now() + delay;
        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(deadline) => return,
                audio = self.rx_audio.recv() => match audio {
                    Some(buffer) => self.recycle(buffer).await,
                    // The mixer has stopped. Sit out the delay anyway: the
                    // session loop sees the closed channel next and stops.
                    None => {
                        tokio::time::sleep_until(deadline).await;
                        return;
                    }
                },
            }
        }
    }

    /// What queued while connecting, cut down to the newest
    /// `RECONNECT_TAIL_CHUNKS`. Older chunks go back to the pool unsent.
    async fn take_recent_audio(&mut self) -> VecDeque<AudioSample> {
        let mut recent = VecDeque::with_capacity(RECONNECT_TAIL_CHUNKS + 1);
        while let Ok(buffer) = self.rx_audio.try_recv() {
            recent.push_back(buffer);
            if recent.len() > RECONNECT_TAIL_CHUNKS
                && let Some(oldest) = recent.pop_front()
            {
                self.recycle(oldest).await;
            }
        }
        recent
    }

    /// Sends `backlog` ahead of the live stream, handing every buffer back to
    /// the pool. False when the session broke on the way.
    async fn send_backlog(
        &self,
        session: &mut Box<dyn SttSession>,
        backlog: VecDeque<AudioSample>,
    ) -> bool {
        let mut sent = true;
        for buffer in backlog {
            if sent {
                sent = session.send(bytemuck::cast_slice(&buffer)).await.is_ok();
            }
            self.recycle(buffer).await;
        }
        sent
    }

    async fn recycle(&self, mut buffer: AudioSample) {
        buffer.clear();
        let _ = self.tx_recycle.send(buffer).await;
    }

    async fn wait_first_packet(&mut self) -> Option<AudioSample> {
        tracing::debug!("Waiting for speech to connect...");
        let mut loudest = 0;

        loop {
            match self.rx_audio.recv().await {
                Some(packet) if !is_silent(&packet, self.vad_threshold) => {
                    return Some(packet);
                }
                Some(mut packet) => {
                    // Nothing has passed the gate yet. Report each new peak, so
                    // a level that never approaches the threshold says the
                    // threshold is wrong for this source — not that the room
                    // is quiet. Logging only on a new maximum keeps it quiet.
                    let level = rms(&packet);
                    if level > loudest {
                        loudest = level;
                        tracing::debug!(
                            "Below the VAD threshold {}, loudest so far {}",
                            self.vad_threshold,
                            loudest
                        );
                    }

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

/// 1, 2, 4 and 8 seconds, then `MAX_BACKOFF` for every attempt after that.
fn backoff(attempt: u32) -> Duration {
    let doublings = attempt.saturating_sub(1).min(4);
    Duration::from_secs(1 << doublings).min(MAX_BACKOFF)
}
