use super::*;
use async_trait::async_trait;
use tokio::time::{sleep, timeout};

struct IdleSession;

#[async_trait]
impl SttSession for IdleSession {
    async fn send(&mut self, _audio: &[u8]) -> Result<(), PipelineError> {
        Ok(())
    }

    async fn recv_event(&mut self) -> Result<PipelineEvent, PipelineError> {
        std::future::pending().await
    }

    fn idle_timeout(&self) -> Option<Duration> {
        Some(Duration::from_millis(50))
    }
}

struct UnusedBackend;

#[async_trait]
impl SttBackend for UnusedBackend {
    async fn connect(&self) -> Result<Box<dyn SttSession>, PipelineError> {
        Err(PipelineError::ConnectionLost)
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

struct HangingBackend;

#[async_trait]
impl SttBackend for HangingBackend {
    async fn connect(&self) -> Result<Box<dyn SttSession>, PipelineError> {
        std::future::pending().await
    }
}

#[test]
fn backoff_doubles_up_to_the_cap() {
    let delays: Vec<u64> = (1..=6).map(|n| backoff(n).as_secs()).collect();
    assert_eq!(delays, [1, 2, 4, 8, 15, 15]);
}

#[test]
fn retries_last_about_ten_minutes() {
    let total: Duration = (1..=MAX_RETRIES).map(backoff).sum();
    assert!(
        (Duration::from_secs(8 * 60)..=Duration::from_secs(12 * 60)).contains(&total),
        "retrying for {total:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn a_hanging_connect_is_given_up_and_retried() {
    let (_tx_audio, rx_audio) = tokio::sync::mpsc::channel(4);
    let (tx_recycle, _rx_recycle) = tokio::sync::mpsc::channel(4);
    let (tx_event, _rx_event) = tokio::sync::mpsc::channel(16);
    let worker = GenericSttWorker::new(
        rx_audio,
        tx_recycle,
        tx_event,
        0,
        0,
        Box::new(HangingBackend),
    );

    let started = tokio::time::Instant::now();
    let result = worker.connect().await;

    assert!(matches!(result, Err(ref e) if e.is_reconnect()));
    assert!(started.elapsed() >= CONNECT_TIMEOUT);
}

#[tokio::test]
async fn a_reconnect_sends_only_the_newest_audio() {
    let (tx_audio, rx_audio) = tokio::sync::mpsc::channel(64);
    let (tx_recycle, mut rx_recycle) = tokio::sync::mpsc::channel(64);
    let (tx_event, _rx_event) = tokio::sync::mpsc::channel(16);
    let mut worker = GenericSttWorker::new(
        rx_audio,
        tx_recycle,
        tx_event,
        0,
        0,
        Box::new(UnusedBackend),
    );

    // Thirty chunks queued during an outage, each tagged with its position.
    for i in 0..30i16 {
        tx_audio.send(vec![i; 8]).await.unwrap();
    }

    let recent = worker.take_recent_audio().await;

    let kept: Vec<i16> = recent.iter().map(|chunk| chunk[0]).collect();
    let expected: Vec<i16> = (30 - RECONNECT_TAIL_CHUNKS as i16..30).collect();
    assert_eq!(kept, expected);

    let mut recycled = 0;
    while rx_recycle.try_recv().is_ok() {
        recycled += 1;
    }
    assert_eq!(recycled, 30 - RECONNECT_TAIL_CHUNKS);
}

#[tokio::test(start_paused = true)]
async fn audio_does_not_pile_up_while_waiting_to_reconnect() {
    let (tx_audio, rx_audio) = tokio::sync::mpsc::channel(64);
    let (tx_recycle, _rx_recycle) = tokio::sync::mpsc::channel(64);
    let (tx_event, _rx_event) = tokio::sync::mpsc::channel(16);
    let mut worker = GenericSttWorker::new(
        rx_audio,
        tx_recycle,
        tx_event,
        0,
        0,
        Box::new(UnusedBackend),
    );

    // The mixer keeps ticking through the outage.
    tokio::spawn(async move {
        while tx_audio.send(vec![0i16; 8]).await.is_ok() {
            sleep(CHUNK_PERIOD).await;
        }
    });

    worker.discard_audio_for(Duration::from_secs(5)).await;

    assert!(worker.take_recent_audio().await.len() <= 1);
}
