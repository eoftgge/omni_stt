use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;
use crate::stt::event::SttEvent;
use crate::stt::factory::create_stt_backend;
use crate::stt::worker::GenericSttWorker;
use crate::transcription::audio::{AudioSample, AudioSession};
use crate::transcription::device::MappableAvailableDevices;
use tokio::sync::mpsc::{Receiver, channel};
use tokio_util::sync::CancellationToken;

pub struct TranscriptionService {
    pub(crate) _audio: AudioSession,
    pub receiver: Receiver<SttEvent>,
    cancel_token: CancellationToken,
    worker_handle: tokio::task::JoinHandle<()>,
    proxy_handle: tokio::task::JoinHandle<()>,
}

impl TranscriptionService {
    pub fn start<F>(
        settings: &SettingsApp,
        devices: &MappableAvailableDevices,
        on_new_event: F,
    ) -> Result<Self, OmniSttErrors>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let device = devices
            .to_output_device(settings.audio.device_id.as_ref())
            .ok_or(OmniSttErrors::NotFoundOutputDevice)?;

        let cancel_token = CancellationToken::new();

        let (tx_worker, mut rx_worker) = channel::<SttEvent>(128);
        let (tx_event, rx_event) = channel::<SttEvent>(128);
        let (tx_audio, rx_audio) = channel::<AudioSample>(2048);
        let (tx_recycle, rx_recycle) = channel::<AudioSample>(2048);

        let audio = AudioSession::open(device.into_inner(), tx_audio, rx_recycle)?;

        let backend = create_stt_backend(&settings.provider)?;
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
            receiver: rx_event,
            cancel_token,
            worker_handle,
            proxy_handle,
        })
    }
}

impl Drop for TranscriptionService {
    fn drop(&mut self) {
        tracing::debug!("Dropping TranscriptionService, cancelling tasks...");
        self.cancel_token.cancel();
        self.worker_handle.abort();
        self.proxy_handle.abort();
    }
}
