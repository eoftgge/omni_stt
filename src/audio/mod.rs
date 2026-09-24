pub mod device;
pub mod mixer;
pub mod resample;
pub mod utils;

use crate::errors::OmniSttErrors;
use crate::event::PipelineEvent;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Error, ErrorKind, Stream, StreamConfig};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::audio::resample::AudioConverter;
use std::time::Duration;

/// Samples per chunk handed downstream.
pub const CHUNK_SAMPLES: usize = 3200;

/// How long one chunk lasts at the 16 kHz target rate. The mixer ticks on it,
/// so the two stay in step by construction rather than by a matching pair of
/// magic numbers.
pub const CHUNK_PERIOD: Duration = Duration::from_millis(CHUNK_SAMPLES as u64 * 1000 / 16_000);

/// Capacity of a chunk buffer. A chunk is handed on once it has reached
/// `CHUNK_SAMPLES`, so the last push overshoots by up to one callback's worth;
/// without the headroom that push would reallocate on the audio thread.
pub const CHUNK_CAPACITY: usize = CHUNK_SAMPLES * 2;

/// Buffers seeded into each capture's pool before its stream starts. A chunk
/// leaves every 200 ms and the mixer returns it on its next tick, so two or
/// three are in flight; the rest cover a mixer stalled for about a second.
pub const POOL_BUFFERS: usize = 8;

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
        tx_event: Sender<PipelineEvent>,
    ) -> Result<Self, OmniSttErrors> {
        let mut accumulator = Vec::with_capacity(CHUNK_CAPACITY);
        // A chunk the channel refused, kept for reuse: freeing it here would
        // cost the audio thread as much as allocating.
        let mut spare: Option<AudioSample> = None;

        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                converter.push(data, &mut accumulator);
                if accumulator.len() < CHUNK_SAMPLES {
                    return;
                }

                // No recycled buffer is available. The mixer is already behind far enough
                // for the bounded pending queues to discard old audio, so retaining this
                // partial chunk would not preserve useful continuity. Keep the converter
                // state advancing and drop the accumulated output without allocating here.
                let Some(mut next) = spare.take().or_else(|| rx_recycle.try_recv().ok()) else {
                    accumulator.clear();
                    return;
                };
                next.clear();
                std::mem::swap(&mut accumulator, &mut next);

                match tx_audio.try_send(next) {
                    Ok(()) => {}
                    Err(TrySendError::Full(chunk)) => {
                        tracing::debug!("Audio buffer is full");
                        spare = Some(chunk);
                    }
                    Err(TrySendError::Closed(chunk)) => {
                        tracing::debug!("Capture channel closed");
                        spare = Some(chunk);
                    }
                }
            },
            audio_error_callback(tx_event),
            None,
        )?;

        Ok(Self::new(stream))
    }

    pub fn play(&self) -> Result<(), Error> {
        self.stream.play()
    }
}

/// cpal calls this from the audio thread and never restarts the stream
/// afterwards, so a single error ends the session. The latch keeps a backend
/// that reports repeatedly from stacking toasts on the user.
fn audio_error_callback(tx_event: Sender<PipelineEvent>) -> impl FnMut(Error) + Send + 'static {
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

        let _ = tx_event.try_send(PipelineEvent::AudioLost(text));
    }
}

/// A dropped or doubled buffer is a hiccup, not a broken stream: the backend
/// keeps delivering audio afterwards, only a few milliseconds are lost. Listed
/// explicitly, so anything we have not seen before is still treated as fatal.
fn is_transient(err: &Error) -> bool {
    matches!(err.kind(), ErrorKind::Xrun)
}
