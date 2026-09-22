use crate::transcription::audio::{AudioSample, CHUNK_SAMPLES};
use std::collections::VecDeque;
use tokio::sync::mpsc::{Receiver, Sender};

/// How far the secondary stream may run ahead before the oldest samples go.
///
/// Two sound cards run on independent crystals and creep apart by tens of
/// milliseconds an hour. Without a ceiling that offset would grow for the
/// whole session; with one, the correction is a handful of samples every few
/// seconds — inaudible, and the recogniser only ever sees a continuous stream.
const MAX_PENDING: usize = CHUNK_SAMPLES * 2;

/// Folds a second capture into the first.
///
/// The primary stream sets the pace: every chunk it delivers is forwarded
/// carrying the same number of samples from whatever the secondary produced
/// meanwhile. Pairing chunk for chunk instead would let the faster side queue
/// up, and latency would climb until its channel filled.
///
/// A secondary that falls silent, stalls or dies contributes nothing and the
/// primary keeps flowing unmixed — losing the microphone must not take the
/// whole session down with it.
pub struct AudioMixer {
    rx_primary: Receiver<AudioSample>,
    rx_secondary: Receiver<AudioSample>,
    tx_audio: Sender<AudioSample>,
    tx_recycle_secondary: Sender<AudioSample>,
    pending: VecDeque<i16>,
    secondary_closed: bool,
}

impl AudioMixer {
    pub fn new(
        rx_primary: Receiver<AudioSample>,
        rx_secondary: Receiver<AudioSample>,
        tx_audio: Sender<AudioSample>,
        tx_recycle_secondary: Sender<AudioSample>,
    ) -> Self {
        Self {
            rx_primary,
            rx_secondary,
            tx_audio,
            tx_recycle_secondary,
            pending: VecDeque::with_capacity(MAX_PENDING),
            secondary_closed: false,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // Drained first, so a primary chunk always mixes against the
                // freshest secondary samples on hand.
                biased;

                secondary = self.rx_secondary.recv(), if !self.secondary_closed => {
                    let Some(mut buffer) = secondary else {
                        tracing::info!("Secondary capture ended, continuing with the primary alone");
                        self.secondary_closed = true;
                        continue;
                    };

                    self.pending.extend(buffer.iter().copied());
                    while self.pending.len() > MAX_PENDING {
                        self.pending.pop_front();
                    }

                    buffer.clear();
                    let _ = self.tx_recycle_secondary.send(buffer).await;
                }

                primary = self.rx_primary.recv() => {
                    let Some(mut buffer) = primary else {
                        return;
                    };

                    for sample in buffer.iter_mut() {
                        let Some(other) = self.pending.pop_front() else {
                            break;
                        };
                        // Saturating on purpose: each side aims for half of
                        // full scale, but their transients can still overlap.
                        *sample = sample.saturating_add(other);
                    }

                    if self.tx_audio.send(buffer).await.is_err() {
                        return;
                    }
                }
            }
        }
    }
}
