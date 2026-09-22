use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;
use crate::stt::event::SttEvent;
use crate::stt::factory::create_stt_backend;
use crate::stt::worker::GenericSttWorker;
use crate::transcription::audio::{self, AudioSample, AudioSession};
use crate::transcription::device::AvailableDevice;
use crate::transcription::mixer::AudioMixer;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_util::sync::CancellationToken;
use crate::transcription::resample::AudioConverter;

const POOL_CAPACITY: usize = 2048;

pub struct TranscriptionService {
    pub(crate) _audio: Vec<AudioSession>,
    pub receiver: Receiver<SttEvent>,
    _worker_handle: tokio::task::JoinHandle<()>,
    cancel_token: CancellationToken,
    proxy_handle: tokio::task::JoinHandle<()>,
}

impl TranscriptionService {
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

        let cancel_token = CancellationToken::new();

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

        let worker_cancel = cancel_token.clone();
        let worker_handle = tokio::spawn(async move {
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

        let proxy_cancel = cancel_token.clone();
        let proxy_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(event) = rx_worker.recv() => {
                        if tx_event.send(event).await.is_err() {
                            break;
                        }
                        on_new_event();
                    }
                    _ = proxy_cancel.cancelled() => {
                        tracing::info!("Proxy task cancelled gracefully");
                        break;
                    }
                    else => break,
                }
            }
        });

        Ok(Self {
            _audio: sessions,
            _worker_handle: worker_handle,
            receiver: rx_event,
            cancel_token,
            proxy_handle,
        })
    }
}

impl Drop for TranscriptionService {
    fn drop(&mut self) {
        tracing::debug!("Dropping TranscriptionService, cancelling tasks...");
        self.cancel_token.cancel();
        self.proxy_handle.abort();
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
