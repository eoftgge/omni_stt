use std::path::{Path, PathBuf};
use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;

pub struct SettingsManager {
    pub settings: SettingsApp,
    pub(self) path: PathBuf,
}

impl SettingsManager {
    pub fn new(path: &str) -> Self {
        let path = PathBuf::from(path);

        let settings = match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<SettingsApp>(&content) {
                Ok(settings) => settings,
                Err(e) => Self::recover_from_broken(&path, &content, e),
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                let settings = SettingsApp::default();
                if let Ok(content) = toml::to_string_pretty(&settings) {
                    let _ = std::fs::write(&path, content);
                }
                settings
            }
            Err(e) => {
                tracing::error!("Failed to read config at {}: {}", path.display(), e);
                SettingsApp::default()
            }
        };
        Self {
            path,
            settings: Self::normalize_provider(settings),
        }
    }

    #[cfg(not(feature = "vosk"))]
    fn normalize_provider(mut settings: SettingsApp) -> SettingsApp {
        use crate::stt::adapters::types::ProviderType;

        if settings.provider.active_type == ProviderType::Vosk {
            settings.provider.active_type = ProviderType::Soniox;
        }
        settings
    }

    #[cfg(feature = "vosk")]
    fn normalize_provider(settings: SettingsApp) -> SettingsApp {
        settings
    }

    pub fn save(&self) -> Result<(), OmniSttErrors> {
        let path = self.path.clone();
        let toml_string = toml::to_string_pretty(&self.settings)?;
        std::fs::write(path, toml_string)?;

        Ok(())
    }

    fn recover_from_broken(path: &Path, content: &str, err: toml::de::Error) -> SettingsApp {
        let backup = path.with_extension("toml.bak");
        match std::fs::write(&backup, content) {
            Ok(_) => tracing::error!(
                "Config at {} is malformed: {}. Backed up to {} and continuing with defaults.",
                path.display(),
                err,
                backup.display()
            ),
            Err(write_err) => tracing::error!(
                "Config at {} is malformed: {}. Could not back up ({}); leaving file untouched.",
                path.display(),
                err,
                write_err
            ),
        }

        SettingsApp::default()
    }
}
