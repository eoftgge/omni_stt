use crate::stt::adapters::types::{ProviderType, SonioxSettings, VoskSettings};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsProvider {
    pub(crate) active_type: ProviderType,
    pub(crate) soniox: SonioxSettings,
    pub(crate) vosk: VoskSettings,
}
