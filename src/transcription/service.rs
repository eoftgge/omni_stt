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

pub struct TranscriptionService {
    pub(crate) _audio_secondary: Option<AudioSession>,
    pub(crate) _audio: AudioSession,
    pub receiver: Receiver<SttEvent>,
    _worker_handle: tokio::task::JoinHandle<()>,
    cancel_token: CancellationToken,
    proxy_handle: tokio::task::JoinHandle<()>,
}

impl TranscriptionService {
    pub async fn start<F>(
        settings: &SettingsApp,
        device: AvailableDevice,
        secondary: Option<AvailableDevice>,
        on_new_event: F,
    ) -> Result<Self, OmniSttErrors>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let cancel_token = CancellationToken::new();

        let (tx_worker, mut rx_worker) = channel::<SttEvent>(128);
        let (tx_event, rx_event) = channel::<SttEvent>(128);
        let (tx_capture, rx_capture) = channel::<AudioSample>(2048);
        let (tx_recycle, rx_recycle) = channel::<AudioSample>(2048);

        let target_peak = if secondary.is_some() {
            audio::MIXED_PEAK
        } else {
            audio::FULL_SCALE_PEAK
        };

        let audio = open_capture(device, target_peak, tx_capture, rx_recycle, tx_worker.clone())?;
        let (rx_audio, audio_secondary) = match secondary {
            Some(second) => {
                let (tx_second, rx_second) = channel::<AudioSample>(2048);
                let (tx_recycle_second, rx_recycle_second) = channel::<AudioSample>(2048);
                let (tx_mixed, rx_mixed) = channel::<AudioSample>(2048);

                let session = open_capture(
                    second,
                    target_peak,
                    tx_second,
                    rx_recycle_second,
                    tx_worker.clone(),
                )?;

                tokio::spawn(
                    AudioMixer::new(rx_capture, rx_second, tx_mixed, tx_recycle_second).run(),
                );

                (rx_mixed, Some(session))
            }
            None => (rx_capture, None),
        };

        let backend = create_stt_backend(&settings.provider).await?;
        let tx_worker_2 = tx_worker.clone();
        let worker = GenericSttWorker::new(
            rx_audio,
            tx_recycle,
            tx_worker_2,
            settings.audio.hangover_chunks,
            settings.audio.vad_threshold,
            backend,
        );

        audio.play()?;
        if let Some(session) = &audio_secondary {
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
            _audio: audio,
            _audio_secondary: audio_secondary,
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
