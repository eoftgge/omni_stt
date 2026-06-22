use crate::stt::languages::LanguageHint;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProviderType {
    #[default]
    Soniox,
    Vosk,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SonioxSettings {
    pub(crate) language_hints: Vec<LanguageHint>,
    pub(crate) context: String,
    pub(crate) api_key: String,
    pub(crate) target_language: LanguageHint,
    pub(crate) enable_translate: bool,
    pub(crate) enable_speakers: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoskSettings {
    pub(crate) path: PathBuf,
}

impl Default for SonioxSettings {
    fn default() -> Self {
        Self {
            language_hints: vec![LanguageHint::default()],
            context: String::from("some kind context"),
            api_key: String::new(),
            target_language: LanguageHint::default(),
            enable_translate: false,
            enable_speakers: true,
        }
    }
}
