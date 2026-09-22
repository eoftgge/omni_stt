pub mod anchor;
pub mod audio;
pub mod general;
pub mod keystore;
pub mod manager;
pub mod provider;
pub mod secret;
pub mod ui;

pub use audio::SettingsAudio;
pub use general::SettingsGeneral;
pub use keystore::KeyStorage;
pub use manager::SettingsManager;
pub use manager::logging_settings;
pub use provider::SettingsProvider;
pub use secret::Secret;
pub use ui::SettingsUI;

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsApp {
    pub general: SettingsGeneral,
    pub(crate) audio: SettingsAudio,
    pub(crate) ui: SettingsUI,
    pub(crate) provider: SettingsProvider,
}
