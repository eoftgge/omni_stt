use crate::settings::Secret;
use crate::settings::languages::LanguageHint;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsProvider {
    pub(crate) active_type: ProviderType,
    pub(crate) soniox: SonioxSettings,
    pub(crate) vosk: VoskSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProviderType {
    #[default]
    Soniox,
    Vosk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SonioxSettings {
    pub(crate) language_hints: Vec<LanguageHint>,
    pub(crate) context: String,
    pub(crate) api_key: Secret<String>,
    pub(crate) target_language: LanguageHint,
    pub(crate) enable_translate: bool,
    pub(crate) enable_speakers: bool,
    /// Keep the API key in omni.toml instead of the system keychain, so the
    /// config works on any computer the folder is copied to.
    pub(crate) store_key_in_file: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct VoskSettings {
    pub(crate) model_path: PathBuf,
    pub(crate) library_path: PathBuf,
}

impl Default for SonioxSettings {
    fn default() -> Self {
        Self {
            language_hints: vec![LanguageHint::default()],
            context: String::from("some kind context"),
            api_key: Secret(String::new()),
            target_language: LanguageHint::default(),
            enable_translate: false,
            enable_speakers: true,
            store_key_in_file: false,
        }
    }
}
