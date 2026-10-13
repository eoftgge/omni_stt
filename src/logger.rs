use crate::settings::general::{SettingsGeneral, TracingLevel};
use std::io::Write;
use std::sync::{Arc, RwLock};
use tracing::Level;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{InitError, RollingFileAppender, Rotation};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{Registry, fmt, reload};

pub const LEVELS: &[TracingLevel] = &[
    TracingLevel::Error,
    TracingLevel::Warn,
    TracingLevel::Info,
    TracingLevel::Debug,
    TracingLevel::Trace,
];

enum Sink {
    Stdout,
    File(NonBlocking),
}

#[derive(Clone)]
struct SwappableWriter(Arc<RwLock<Sink>>);

pub struct TracingControl {
    level_handle: reload::Handle<LevelFilter, Registry>,
    sink: Arc<RwLock<Sink>>,
    guard: Option<WorkerGuard>,
    applied: SettingsGeneral,
}

fn file_sink() -> Result<(Sink, WorkerGuard), InitError> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("omni.log")
        .build("logs")?;
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);
    Ok((Sink::File(non_blocking), guard))
}

fn make_sink(log_to_file: bool) -> (Sink, Option<WorkerGuard>, Option<InitError>) {
    if !log_to_file {
        return (Sink::Stdout, None, None);
    }
    match file_sink() {
        Ok((sink, guard)) => (sink, Some(guard), None),
        Err(e) => (Sink::Stdout, None, Some(e)),
    }
}

pub fn setup_tracing(level: Level, log_to_file: bool, applied: SettingsGeneral) -> TracingControl {
    let (sink, guard, error) = make_sink(log_to_file);
    let sink = Arc::new(RwLock::new(sink));
    let (level_layer, level_handle) = reload::Layer::new(LevelFilter::from_level(level));

    tracing_subscriber::registry()
        .with(level_layer)
        .with(
            fmt::layer()
                .with_writer(SwappableWriter(Arc::clone(&sink)))
                .with_ansi(false)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    if let Some(e) = error {
        tracing::error!("Cannot write logs to a file, using stdout: {e}");
    }

    TracingControl {
        level_handle,
        sink,
        guard,
        applied,
    }
}

impl From<TracingLevel> for Level {
    fn from(level: TracingLevel) -> Self {
        match level {
            TracingLevel::Error => Self::ERROR,
            TracingLevel::Warn => Self::WARN,
            TracingLevel::Info => Self::INFO,
            TracingLevel::Debug => Self::DEBUG,
            TracingLevel::Trace => Self::TRACE,
        }
    }
}

impl TracingControl {
    pub fn apply(&mut self, general: &SettingsGeneral) {
        if let Err(e) = self
            .level_handle
            .modify(|f| *f = LevelFilter::from_level(general.level()))
        {
            tracing::error!("Failed to change log level: {e}");
        }
        self.set_log_to_file(general.log_to_file);
    }

    pub fn sync(&mut self, general: &SettingsGeneral) {
        if self.applied == *general {
            return;
        }
        self.apply(general);
        self.applied = general.clone();
    }

    fn set_log_to_file(&mut self, enabled: bool) {
        let is_file = matches!(
            &*self.sink.read().expect("sink lock poisoned"),
            Sink::File(_)
        );
        if is_file == enabled {
            return;
        }

        let (new_sink, new_guard, error) = make_sink(enabled);
        *self.sink.write().expect("sink lock poisoned") = new_sink;
        self.guard = new_guard;
        // Only after the write lock is released: logging takes the read lock.
        if let Some(e) = error {
            tracing::error!("Cannot write logs to a file, using stdout: {e}");
        }
    }
}

impl<'a> MakeWriter<'a> for SwappableWriter {
    type Writer = Box<dyn Write + Send>;

    fn make_writer(&'a self) -> Self::Writer {
        match &*self.0.read().expect("sink lock poisoned") {
            Sink::Stdout => Box::new(std::io::stdout()),
            Sink::File(nb) => Box::new(nb.clone()),
        }
    }
}
