use super::*;
use async_trait::async_trait;
use tokio::time::timeout;

struct IdleSession;

#[async_trait]
impl SttSession for IdleSession {
    async fn send(&mut self, _audio: &[u8]) -> Result<(), SttError> {
        Ok(())
    }

    async fn recv_event(&mut self) -> Result<SttEvent, SttError> {
        std::future::pending().await
    }

    fn idle_timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(50))
    }
}

struct UnusedBackend;

#[async_trait]
impl SttBackend for UnusedBackend {
    async fn connect(&self) -> Result<Box<dyn SttSession>, SttError> {
        Err(SttError::ConnectionLost)
    }
}

#[tokio::test]
async fn session_closes_itself_before_the_server_does() {
    let (tx_audio, rx_audio) = tokio::sync::mpsc::channel(4);
    let (tx_recycle, _rx_recycle) = tokio::sync::mpsc::channel(4);
    let (tx_event, _rx_event) = tokio::sync::mpsc::channel(16);

    let mut worker = GenericSttWorker::new(
        rx_audio,
        tx_recycle,
        tx_event,
        0,
        0,
        Box::new(UnusedBackend),
    );

    let mut session: Box<dyn SttSession> = Box::new(IdleSession);
    let action = worker.run_session_loop(&mut session).await;

    assert!(matches!(action, StreamAction::Idle));
    drop(tx_audio);
}

#[tokio::test]
async fn sent_audio_pushes_the_idle_deadline_back() {
    let (tx_audio, rx_audio) = tokio::sync::mpsc::channel(4);
    let (tx_recycle, mut rx_recycle) = tokio::sync::mpsc::channel(4);
    let (tx_event, _rx_event) = tokio::sync::mpsc::channel(16);
    let mut worker = GenericSttWorker::new(
        rx_audio,
        tx_recycle,
        tx_event,
        0,
        0,
        Box::new(UnusedBackend),
    );

    tokio::spawn(async move { while rx_recycle.recv().await.is_some() {} });
    tokio::spawn(async move {
        while tx_audio.send(vec![0i16; 8]).await.is_ok() {
            sleep(Duration::from_millis(20)).await;
        }
    });

    let mut session: Box<dyn SttSession> = Box::new(IdleSession);
    let outcome = timeout(
        Duration::from_millis(300),
        worker.run_session_loop(&mut session),
    )
    .await;

    assert!(
        outcome.is_err(),
        "сессия закрылась по простою, хотя аудио шло каждые 20 мс при таймауте 50 мс"
    );
}
