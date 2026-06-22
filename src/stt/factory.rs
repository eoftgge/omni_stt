use crate::settings::SettingsProvider;
use crate::stt::adapters::soniox::SonioxBackend;
use crate::stt::backend::SttBackend;
use crate::stt::event::SttError;

use crate::stt::adapters::soniox::request::create_request;
use crate::stt::adapters::types::ProviderType;
use crate::stt::adapters::vosk::VoskBackend;

pub fn create_stt_backend(
    settings_provider: &SettingsProvider,
) -> Result<Box<dyn SttBackend>, SttError> {
    match settings_provider.active_type {
        ProviderType::Soniox => {
            let request = create_request(settings_provider.soniox.to_owned()).map_err(|e| {
                SttError::FatalAPIError(format!("Failed to build Soniox request: {}", e))
            })?;
            Ok(Box::new(SonioxBackend::new(request)))
        }
        ProviderType::Vosk => Ok(Box::new(VoskBackend::new(
            settings_provider.vosk.path.to_owned(),
        )?)),
    }
}
