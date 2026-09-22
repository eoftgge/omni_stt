use crate::logger::TracingLevel;
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
