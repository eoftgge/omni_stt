use serde::{Deserialize, Serialize};
use crate::audio::device::AudioSource;

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsAudio {
    pub(crate) sources: Vec<AudioSource>,
    pub(crate) hangover_chunks: usize,
    pub(crate) vad_threshold: u32,
}

impl Default for SettingsAudio {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            hangover_chunks: 15,
            vad_threshold: 500,
        }
    }
}
