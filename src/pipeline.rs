
use crate::audio;
use crate::audio::device::AvailableDevice;
use crate::audio::mixer::AudioMixer;
use crate::audio::resample::AudioConverter;
use crate::audio::{AudioSample, AudioSession};
use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;
use crate::stt::event::SttEvent;
use crate::stt::factory::create_stt_backend;
use crate::stt::worker::GenericSttWorker;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::sync::{CancellationToken, DropGuard};

const POOL_CAPACITY: usize = 2048;

/// A running capture-to-subtitles pipeline. Read events from it, drop it to
/// stop everything.
///
/// Shutdown rides on the field order below, which is why the fields are
/// arranged this way rather than by importance.
pub struct Pipeline {
    /// Dropped first. Stopping the capture streams closes the channels feeding
    /// the mixer, and that is the only way the mixer learns to stop — it
    /// watches no token of its own and ends on its next tick.
    _captures: Vec<AudioSession>,

    pub receiver: Receiver<SttEvent>,

    /// Dropped last, cancelling the worker and the event proxy. Both already
    /// select on this token, so there is nothing left to abort by hand. The
    /// worker's future dies mid-await, which closes the provider socket
    /// abruptly rather than with a close frame — deliberate; the server sees
    /// the dropped connection immediately either way.
    _shutdown: DropGuard,
}

impl Pipeline {
    pub async fn start<F>(
        settings: &SettingsApp,
        devices: Vec<AvailableDevice>,
        on_new_event: F,
    ) -> Result<Self, OmniSttErrors>
    where
        F: Fn() + Send + Sync + 'static,
    {
        if devices.is_empty() {
            return Err(OmniSttErrors::NotFoundAudioDevice);
        }

        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let proxy_cancel = cancel.clone();

        let (tx_worker, mut rx_worker) = channel::<SttEvent>(128);
        let (tx_event, rx_event) = channel::<SttEvent>(128);
        let (tx_recycle, rx_recycle) = channel::<AudioSample>(POOL_CAPACITY);
        let (tx_mixed, rx_mixed) = channel::<AudioSample>(POOL_CAPACITY);

        // Independent sources add in power, not amplitude: N of them land
        // around √N times one, not N times. Dividing by N would quietly rob
        // every source of level for a collision that mostly does not happen,
        // and the saturating add in the mixer is there for when it does.
        let target_peak = audio::FULL_SCALE_PEAK / (devices.len() as f32).sqrt();

        let mut sessions = Vec::with_capacity(devices.len());
        let mut inputs = Vec::with_capacity(devices.len());

        for device in devices {
            let (tx_capture, rx_capture) = channel::<AudioSample>(POOL_CAPACITY);
            let (tx_recycle_source, rx_recycle_source) = channel::<AudioSample>(POOL_CAPACITY);

            sessions.push(open_capture(
                device,
                target_peak,
                tx_capture,
                rx_recycle_source,
                tx_worker.clone(),
            )?);
            inputs.push((rx_capture, tx_recycle_source));
        }

        tokio::spawn(AudioMixer::new(inputs, rx_recycle, tx_mixed).run());

        let backend = create_stt_backend(&settings.provider).await?;
        let worker = GenericSttWorker::new(
            rx_mixed,
            tx_recycle,
            tx_worker.clone(),
            settings.audio.hangover_chunks,
            settings.audio.vad_threshold,
            backend,
        );

        for session in &sessions {
            session.play()?;
        }

        tokio::spawn(async move {
            tokio::select! {
                res = worker.run() => {
                    if let Err(e) = res {
                        tracing::error!("Worker error: {:?}", e);
                        let _ = tx_worker.send(SttEvent::Error(e)).await;
                    }
                }
                _ = worker_cancel.cancelled() => {
                    tracing::info!("Worker cancelled gracefully via token");
                }
            }
        });
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = rx_worker.recv() => {
                        let Some(event) = event else {
                            break;
                        };
                        if tx_event.send(event).await.is_err() {
                            break;
                        }
                        on_new_event();
                    }
                    _ = proxy_cancel.cancelled() => {
                        tracing::info!("Proxy task cancelled gracefully");
                        break;
                    }
                }
            }
        });
        Ok(Self {
            _captures: sessions,
            receiver: rx_event,
            _shutdown: cancel.drop_guard(),
        })
    }
}

fn open_capture(
    device: AvailableDevice,
    target_peak: f32,
    tx_audio: Sender<AudioSample>,
    rx_recycle: Receiver<AudioSample>,
    tx_event: Sender<SttEvent>,
) -> Result<AudioSession, OmniSttErrors> {
    let config = device.stream_config()?;

    tracing::info!(
        "Capturing {:?} '{}' at {} Hz, {} ch",
        device.kind(),
        device.name(),
        config.sample_rate,
        config.channels
    );
    let converter =
        AudioConverter::new(config.sample_rate, config.channels).with_target_peak(target_peak);

    AudioSession::open(
        device.into_inner(),
        config,
        converter,
        tx_audio,
        rx_recycle,
        tx_event,
    )
}
