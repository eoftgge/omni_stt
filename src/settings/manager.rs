use crate::errors::OmniSttErrors;
use crate::settings::SettingsApp;
use crate::settings::general::SettingsGeneral;
use crate::settings::secret::Secret;
use crate::settings::{KeyStorage, keystore};
use std::io::Write;
use std::path::{Path, PathBuf};

fn write_atomic(path: &Path, content: &str) -> std::io::Result<()> {
    let tmp = path.with_extension(format!("toml.{}.tmp", std::process::id()));

    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&tmp, path)
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

pub fn logging_settings(path: &str) -> SettingsGeneral {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| toml::from_str::<SettingsApp>(&content).ok())
        .map(|settings| settings.general)
        .unwrap_or_default()
}

pub struct SettingsManager {
    pub(crate) key_storage: KeyStorage,
    pub settings: SettingsApp,
    path: PathBuf,
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
        let stored = keystore::load();
        let key_storage = match &stored {
            Ok(_) => KeyStorage::Keyring,
            Err(e) => {
                tracing::warn!("System key storage unavailable: {e}");
                KeyStorage::PlainFile {
                    reason: e.to_string(),
                }
            }
        };

        let keyring_in_use = matches!(key_storage, KeyStorage::Keyring)
            && !settings.provider.soniox.store_key_in_file;
        // In keyring mode every save blanks the key in the file, so a key found
        // there was written by hand and is newer than the keychain's.
        let migrate = keyring_in_use && !file_key.is_empty();
        settings.provider.soniox.api_key = match stored {
            Ok(Some(key)) if keyring_in_use && !migrate => Secret(key),
            _ => file_key,
        };

        let manager = Self {
            path,
            settings,
            key_storage,
        };

        if migrate {
            match manager.save() {
                Ok(()) => {
                    tracing::info!("API key moved to the system keychain");
                    manager.remove_backup();
                }
                Err(e) => tracing::error!("Failed to move API key to the system keychain: {e}"),
            }
        }

        manager
    }

    pub fn key_storage(&self) -> &KeyStorage {
        &self.key_storage
    }

    pub fn save(&self) -> Result<(), OmniSttErrors> {
        let mut to_write = self.settings.clone();

        match self.key_storage {
            KeyStorage::Keyring if !self.settings.provider.soniox.store_key_in_file => {
                let key = &self.settings.provider.soniox.api_key;
                if key.is_empty() {
                    let _ = keystore::delete();
                } else {
                    keystore::store(key)?;
                }
                to_write.provider.soniox.api_key = Default::default();
            }
            // Kept in the file by choice: don't leave a stale second copy
            // behind in the keychain.
            KeyStorage::Keyring => {
                let _ = keystore::delete();
            }
            KeyStorage::PlainFile { .. } => {}
        }

        write_atomic(&self.path, &toml::to_string_pretty(&to_write)?)?;
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
