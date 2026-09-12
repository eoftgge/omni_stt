use serde::{Deserialize, Serialize};
use tracing::Level;
use crate::settings::SettingsGeneral;
use std::io::Write;
use std::sync::{Arc, RwLock};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
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
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TracingLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

fn make_sink(log_to_file: bool) -> (Sink, Option<WorkerGuard>) {
    if log_to_file {
        let appender = tracing_appender::rolling::daily("logs", "omni.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(appender);
        (Sink::File(non_blocking), Some(guard))
    } else {
        (Sink::Stdout, None)
    }
}

pub fn setup_tracing(level: Level, log_to_file: bool) -> TracingControl {
    let (sink, guard) = make_sink(log_to_file);
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

    TracingControl { level_handle, sink, guard }
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

impl std::fmt::Display for TracingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            TracingLevel::Error => "ERROR",
            TracingLevel::Warn => "WARN",
            TracingLevel::Info => "INFO",
            TracingLevel::Debug => "DEBUG",
            TracingLevel::Trace => "TRACE",
        };
        write!(f, "{}", level)
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
        self.set_log_to_file(general.log_to_file());
    }

    fn set_log_to_file(&mut self, enabled: bool) {
        let is_file = matches!(&*self.sink.read().expect("sink lock poisoned"), Sink::File(_));
        if is_file == enabled {
            return;
        }

        let (new_sink, new_guard) = make_sink(enabled);
        *self.sink.write().expect("sink lock poisoned") = new_sink;
        self.guard = new_guard;
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