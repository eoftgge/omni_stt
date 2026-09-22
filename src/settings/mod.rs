pub mod keystore;
pub mod manager;
pub mod secret;
pub mod ui;
pub mod audio;
pub mod provider;
pub mod general;

pub use keystore::KeyStorage;
pub use manager::SettingsManager;
pub use manager::logging_settings;
pub use secret::Secret;
pub use audio::SettingsAudio;
pub use general::SettingsGeneral;
pub use provider::SettingsProvider;
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
