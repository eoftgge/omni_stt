use crate::errors::OmniSttErrors;
use crate::transcription::utils::convert_audio_chunk;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, Stream};
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
    ) -> Result<Self, OmniSttErrors> {
        let config = device.default_output_config()?.config();
        let target_samples = 3200;
        let mut accumulator = Vec::with_capacity(target_samples);

        let channel = config.channels;
        let sample_rate = config.sample_rate;
        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                convert_audio_chunk(data, &mut accumulator, channel, sample_rate);
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
            |err| {
                tracing::error!("Error in audio callback: {}", err);
            },
            None,
        )?;

        Ok(Self::new(stream))
    }

    pub fn play(&self) -> Result<(), cpal::Error> {
        self.stream.play()
    }
}
