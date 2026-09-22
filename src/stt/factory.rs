use crate::settings::provider::SettingsProvider;
use crate::stt::adapters::soniox::SonioxBackend;
use crate::stt::backend::SttBackend;
use crate::stt::event::SttError;

use crate::stt::adapters::soniox::request::create_request;
use crate::stt::adapters::types::ProviderType;
use crate::stt::adapters::vosk::VoskBackend;

pub async fn create_stt_backend(
    settings_provider: &SettingsProvider,
) -> Result<Box<dyn SttBackend>, SttError> {
    match settings_provider.active_type {
        ProviderType::Soniox => {
            let request = create_request(settings_provider.soniox.to_owned()).map_err(|e| {
                SttError::FatalAPIError(format!("Failed to build Soniox request: {}", e))
            })?;
            Ok(Box::new(SonioxBackend::new(request)))
        }
        ProviderType::Vosk => {
            let vosk = &settings_provider.vosk;
            let library_path =
                (!vosk.library_path.as_os_str().is_empty()).then(|| vosk.library_path.clone());

            Ok(Box::new(
                VoskBackend::new(vosk.model_path.to_owned(), library_path).await?,
            ))
        }
    }
}
