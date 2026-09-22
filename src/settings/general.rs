use serde::{Deserialize, Serialize};
use tracing::Level;

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsGeneral {
    pub(crate) level: TracingLevel,
    pub log_to_file: bool,
    pub save_transcripts: bool,
}

impl Default for SettingsGeneral {
    fn default() -> Self {
        Self {
            level: TracingLevel::Info,
            log_to_file: false,
            save_transcripts: false,
        }
    }
}

impl SettingsGeneral {
    pub fn level(&self) -> Level {
        Level::from(self.level)
    }
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
