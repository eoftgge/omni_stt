use crate::settings::SettingsProvider;
use crate::stt::adapters::soniox::SonioxAdapter;
use crate::stt::event::SttError;
use crate::stt::provider::SttProvider;

use crate::stt::adapters::soniox::request::create_request;
use crate::stt::adapters::types::ProviderType;
use crate::stt::adapters::vosk::VoskAdapter;

pub fn create_stt_provider(settings_provider: &SettingsProvider) -> Result<Box<dyn SttProvider>, SttError> {
    match settings_provider.active_type {
        ProviderType::Soniox => {
            let request = create_request(settings_provider.soniox.clone())
                .map_err(|e| SttError::FatalAPIError(format!("Failed to build Soniox request: {}", e)))?;
            Ok(Box::new(SonioxAdapter::new(request)))
        },
        ProviderType::Vosk => {
            Ok(Box::new(VoskAdapter::new(settings_provider.vosk.path.clone())))
        }
    }
}
