use super::types::{SonioxTranscriptionRequest, SonioxTranslationObject};
use crate::errors::OmniSttErrors;
use crate::settings::secret::Secret;
use crate::stt::adapters::soniox::MODEL;
use crate::stt::adapters::types::SonioxSettings;
use std::sync::Arc;

pub(crate) fn create_request(
    settings: SonioxSettings,
) -> Result<SonioxTranscriptionRequest, OmniSttErrors> {
    let mut request = SonioxTranscriptionRequest {
        api_key: Secret(Arc::from(settings.api_key.0)),
        model: MODEL,
        audio_format: "pcm_s16le",
        sample_rate: Some(16000),
        num_channels: Some(1),
        context: Some(Arc::from(settings.context)),
        language_hints: Arc::from(settings.language_hints),
        enable_speaker_diarization: Some(settings.enable_speakers),
        ..Default::default()
    };
    if settings.enable_translate {
        request.translation = Some(SonioxTranslationObject {
            r#type: "one_way",
            target_language: Some(settings.target_language),
            ..Default::default()
        });
    }

    Ok(request)
}
