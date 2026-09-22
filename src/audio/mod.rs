pub mod device;
pub mod mixer;
pub mod resample;

use crate::errors::OmniSttErrors;
use crate::stt::event::SttEvent;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Error, ErrorKind, Stream, StreamConfig};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};

use std::time::Duration;
use crate::audio::resample::AudioConverter;

/// Samples per chunk handed downstream.
pub const CHUNK_SAMPLES: usize = 3200;

/// How long one chunk lasts at the 16 kHz target rate. The mixer ticks on it,
/// so the two stay in step by construction rather than by a matching pair of
/// magic numbers.
pub const CHUNK_PERIOD: Duration = Duration::from_millis(CHUNK_SAMPLES as u64 * 1000 / 16_000);

/// What a lone capture aims for. With several, each takes a share of it.
pub const FULL_SCALE_PEAK: f32 = 0.9;

pub type AudioSample = Vec<i16>;

pub struct AudioSession {
    stream: Stream,
}

impl AudioSession {
    pub fn new(stream: Stream) -> Self {
        Self { stream }
    }

    pub fn open(
        device: Device,
        config: StreamConfig,
        mut converter: AudioConverter,
        tx_audio: Sender<AudioSample>,
        mut rx_recycle: Receiver<AudioSample>,
        tx_event: Sender<SttEvent>,
    ) -> Result<Self, OmniSttErrors> {
        let target_samples = 3200;
        let mut accumulator = Vec::with_capacity(target_samples);

        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                converter.push(data, &mut accumulator);
                if accumulator.len() >= target_samples {
                    let mut next_accumulator = match rx_recycle.try_recv() {
                        Ok(mut recycled) => {
                            recycled.clear();
                            recycled
                        }
                        Err(_) => Vec::with_capacity(target_samples),
                    };

                    std::mem::swap(&mut accumulator, &mut next_accumulator);
                    let samples = next_accumulator;

                    match tx_audio.try_send(samples) {
                        Ok(_) => {}
                        Err(TrySendError::Full(_)) => {
                            tracing::debug!("Audio buffer is full");
                        }
                        Err(TrySendError::Closed(_)) => {
                            tracing::debug!("Capture channel closed");
                        }
                    }
                }
            },
            audio_error_callback(tx_event),
            None,
        )?;

        Ok(Self::new(stream))
    }

    pub fn play(&self) -> Result<(), cpal::Error> {
        self.stream.play()
    }
}

/// cpal calls this from the audio thread and never restarts the stream
/// afterwards, so a single error ends the session. The latch keeps a backend
/// that reports repeatedly from stacking toasts on the user.
fn audio_error_callback(tx_event: Sender<SttEvent>) -> impl FnMut(Error) + Send + 'static {
    let mut reported = false;

    move |err| {
        // Must return before the latch: a hiccup that armed it would silence
        // a real device loss later in the same session.
        if is_transient(&err) {
            tracing::warn!("Audio glitch, continuing: {}", err);
            return;
        }

        tracing::error!("Audio capture stopped: {}", err);
        if reported {
            return;
        }
        reported = true;

        let text = match err.kind() {
            ErrorKind::DeviceNotAvailable => "Audio device disconnected".to_string(),
            _ => format!("Audio capture failed: {err}"),
        };

        let _ = tx_event.try_send(SttEvent::AudioLost(text));
    }
}

/// A dropped or doubled buffer is a hiccup, not a broken stream: the backend
/// keeps delivering audio afterwards, only a few milliseconds are lost. Listed
/// explicitly, so anything we have not seen before is still treated as fatal.
fn is_transient(err: &Error) -> bool {
    matches!(err.kind(), ErrorKind::Xrun)
}
