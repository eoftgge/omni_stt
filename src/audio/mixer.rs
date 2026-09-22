use std::collections::VecDeque;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::MissedTickBehavior;
use crate::audio::{AudioSample, CHUNK_PERIOD, CHUNK_SAMPLES};

/// How far a source may run ahead before its oldest samples are dropped.
///
/// Sound cards run on independent crystals and creep against each other, and
/// against our own timer, by tens of milliseconds an hour. Without a ceiling
/// that offset would grow for the whole session; with one, the correction is a
/// handful of samples every few seconds — inaudible, and the recogniser only
/// ever sees a continuous stream.
const MAX_PENDING: usize = CHUNK_SAMPLES * 2;

struct Source {
    rx: Receiver<AudioSample>,
    tx_recycle: Sender<AudioSample>,
    pending: VecDeque<i16>,
    closed: bool,
}

/// Sums any number of captures into the single stream the worker reads.
///
/// The mixer runs on its own clock instead of following one of the sources,
/// and that is the whole point. A WASAPI loopback stops producing packets
/// entirely while nothing is playing through that device, so a source-driven
/// mixer would freeze the microphone too every time the speakers went quiet.
/// On a tick every source is drained for whatever it has, short ones are
/// padded with silence, and the chunk goes out regardless.
pub struct AudioMixer {
    sources: Vec<Source>,
    rx_recycle: Receiver<AudioSample>,
    tx_audio: Sender<AudioSample>,
}

impl AudioMixer {
    pub fn new(
        inputs: Vec<(Receiver<AudioSample>, Sender<AudioSample>)>,
        rx_recycle: Receiver<AudioSample>,
        tx_audio: Sender<AudioSample>,
    ) -> Self {
        let sources = inputs
            .into_iter()
            .map(|(rx, tx_recycle)| Source {
                rx,
                tx_recycle,
                pending: VecDeque::with_capacity(MAX_PENDING),
                closed: false,
            })
            .collect();

        Self {
            sources,
            rx_recycle,
            tx_audio,
        }
    }

    pub async fn run(mut self) {
        let mut tick = tokio::time::interval(CHUNK_PERIOD);
        // Never burst to catch up after a stall: those chunks would carry no
        // more audio than one, and the worker would just see a gap of silence
        // arrive all at once.
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tick.tick().await;

            if self.drain_sources() {
                tracing::info!("Every capture source has ended, stopping the mixer");
                return;
            }

            let mut chunk = self.take_buffer();
            chunk.clear();
            chunk.resize(CHUNK_SAMPLES, 0);

            for source in &mut self.sources {
                for sample in chunk.iter_mut() {
                    let Some(value) = source.pending.pop_front() else {
                        // This source came up short for this tick; the rest of
                        // the chunk simply carries nothing from it.
                        break;
                    };
                    // Saturating on purpose: each source aims for its share of
                    // full scale, but their transients can still overlap.
                    *sample = sample.saturating_add(value);
                }
            }

            if self.tx_audio.send(chunk).await.is_err() {
                return;
            }
        }
    }

    /// Pulls everything each source has produced since the last tick.
    ///
    /// `try_recv` throughout: the tick decides when a chunk goes out, so
    /// nothing in here may wait on a source. Returns true once every source
    /// has ended and there is no reason to keep ticking.
    fn drain_sources(&mut self) -> bool {
        for source in &mut self.sources {
            loop {
                match source.rx.try_recv() {
                    Ok(mut buffer) => {
                        source.pending.extend(buffer.iter().copied());
                        while source.pending.len() > MAX_PENDING {
                            source.pending.pop_front();
                        }

                        buffer.clear();
                        let _ = source.tx_recycle.try_send(buffer);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        if !source.closed {
                            tracing::info!("A capture source ended, the rest keep going");
                            source.closed = true;
                        }
                        break;
                    }
                }
            }
        }

        self.sources.iter().all(|source| source.closed)
    }

    /// Reuses a buffer the worker has finished with, or makes one when the
    /// pool has not come round yet. Allocating here is merely wasteful — this
    /// runs on a tokio task, not on an audio thread.
    fn take_buffer(&mut self) -> AudioSample {
        self.rx_recycle
            .try_recv()
            .unwrap_or_else(|_| Vec::with_capacity(CHUNK_SAMPLES))
    }
}
