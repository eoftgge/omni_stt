use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;
use crate::settings::keystore::{self, KeyStorage};
use crate::settings::secret::Secret;
use std::path::{Path, PathBuf};

pub struct SettingsManager {
    pub settings: SettingsApp,
    path: PathBuf,
    key_storage: KeyStorage,
}

impl SettingsManager {
    pub fn new(path: &str) -> Self {
        let path = PathBuf::from(path);

        let mut settings = match std::fs::read_to_string(&path) {
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

        let file_key = std::mem::take(&mut settings.provider.soniox.api_key);
        let mut migrated = false;

        let key_storage = match keystore::load() {
            Ok(Some(key)) => {
                settings.provider.soniox.api_key = Secret(key);
                KeyStorage::Keyring
            }
            Ok(None) => {
                if !file_key.is_empty() {
                    match keystore::store(&file_key) {
                        Ok(()) => {
                            migrated = true;
                            tracing::info!("API key moved to the system keychain");
                        }
                        Err(e) => tracing::error!("Failed to migrate API key: {e}"),
                    }
                }
                settings.provider.soniox.api_key = file_key;
                KeyStorage::Keyring
            }
            Err(e) => {
                tracing::warn!("System key storage unavailable: {e}");
                settings.provider.soniox.api_key = file_key;
                KeyStorage::PlainFile {
                    reason: e.to_string(),
                }
            }
        };

        let manager = Self {
            path,
            settings: Self::normalize_provider(settings),
            key_storage,
        };

        if migrated {
            if let Err(e) = manager.save() {
                tracing::error!("Failed to scrub plaintext key from config: {e}");
            }
            manager.remove_backup();
        }

        manager
    }

    pub fn key_storage(&self) -> &KeyStorage {
        &self.key_storage
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
        let mut to_write = self.settings.clone();

        if matches!(self.key_storage, KeyStorage::Keyring) {
            let key = &self.settings.provider.soniox.api_key;
            if key.is_empty() {
                let _ = keystore::delete();
            } else {
                keystore::store(key)?;
            }
            to_write.provider.soniox.api_key = Default::default();
        }

        std::fs::write(&self.path, toml::to_string_pretty(&to_write)?)?;
        Ok(())
    }

    fn remove_backup(&self) {
        let backup = self.path.with_extension("toml.bak");
        if backup.exists() {
            tracing::warn!(
                "Removing {} — it may contain a plaintext API key",
                backup.display()
            );
            let _ = std::fs::remove_file(&backup);
        }
    }

    fn recover_from_broken(path: &Path, content: &str, err: toml::de::Error) -> SettingsApp {
        let backup = path.with_extension("toml.bak");
        match std::fs::write(&backup, content) {
            Ok(_) => tracing::error!(
                "Config at {} is malformed: {}. Backed up to {} (may contain a plaintext API key) and continuing with defaults.",
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