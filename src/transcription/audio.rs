use crate::errors::OmniSttErrors;
use crate::transcription::resample::AudioConverter;
use crate::stt::event::SttEvent;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream, Error, ErrorKind};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};

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
        tx_audio: Sender<AudioSample>,
        mut rx_recycle: Receiver<AudioSample>,
        tx_event: Sender<SttEvent>,
    ) -> Result<Self, OmniSttErrors> {
        let config = device.default_output_config()?.config();
        let target_samples = 3200;
        let mut accumulator = Vec::with_capacity(target_samples);

        let channels = config.channels;
        let sample_rate = config.sample_rate;
        let mut converter = AudioConverter::new(sample_rate, channels);
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
        tracing::error!("Error in audio callback: {}", err);
        if reported {
            return;
        }
        reported = true;

        let text = match &err.kind() {
            ErrorKind::DeviceNotAvailable => "Audio device disconnected".to_string(),
            other => format!("Audio capture failed: {other}"),
        };

        // try_send, never send: blocking the audio thread would stall capture,
        // and the log line above has already recorded the error regardless.
        let _ = tx_event.try_send(SttEvent::AudioLost(text));
    }
}
