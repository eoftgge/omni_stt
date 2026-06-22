use crate::errors::OmniSttErrors;
use crate::logger::TracingLevel;
use crate::stt::adapters::types::{ProviderType, SonioxSettings, VoskSettings};
use crate::transcription::device::SettingDeviceId;
use eframe::egui::{Align2, Color32, Vec2, vec2};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::Level;

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsUI {
    pub(crate) max_blocks: usize,
    pub(crate) offset: (f32, f32),
    pub(crate) anchor: usize,
    pub(crate) font_size: usize,
    pub(crate) background_color: [u8; 4],
    pub(crate) text_color: [u8; 3],
    pub(crate) enable_high_priority: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsAudio {
    pub(crate) device_id: Option<SettingDeviceId>,
    pub(crate) hangover_chunks: usize,
    pub(crate) vad_threshold: u32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsProvider {
    pub(crate) active_type: ProviderType,
    pub(crate) soniox: SonioxSettings,
    pub(crate) vosk: VoskSettings,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SettingsGeneral {
    pub level: TracingLevel,
    pub log_to_file: bool,
}

#[derive(Default, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct SettingsApp {
    pub general: SettingsGeneral,
    pub(crate) audio: SettingsAudio,
    pub(crate) ui: SettingsUI,
    pub(crate) provider: SettingsProvider,
}

pub struct SettingsManager {
    pub settings: SettingsApp,
    pub(self) path: PathBuf,
}

impl SettingsUI {
    pub const DEFAULT_BACKGROUND_COLOR: [u8; 4] = [0, 0, 0, 150];
    pub const DEFAULT_TEXT_COLOR: [u8; 3] = [255, 255, 0]; // yellow
}

impl Default for SettingsUI {
    fn default() -> Self {
        Self {
            enable_high_priority: true,
            offset: (0.0, -30.0),
            anchor: 7,
            font_size: 21,
            background_color: Self::DEFAULT_BACKGROUND_COLOR,
            text_color: Self::DEFAULT_TEXT_COLOR,
            max_blocks: 3,
        }
    }
}

impl Default for SettingsAudio {
    fn default() -> Self {
        Self {
            device_id: None,
            hangover_chunks: 15,
            vad_threshold: 500,
        }
    }
}

impl Default for SettingsGeneral {
    fn default() -> Self {
        Self {
            level: TracingLevel::Info,
            log_to_file: false,
        }
    }
}

impl Default for SettingsProvider {
    fn default() -> Self {
        Self {
            active_type: ProviderType::Soniox,
            soniox: SonioxSettings::default(),
            vosk: VoskSettings::default(),
        }
    }
}

impl SettingsUI {
    pub fn text_color(&self) -> Color32 {
        let color = self.text_color;
        Color32::from_rgb(color[0], color[1], color[2])
    }

    pub fn background_color(&self) -> Color32 {
        let color = self.background_color;
        Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3])
    }

    pub fn get_anchor(&self) -> (Align2, Vec2) {
        let align = match self.anchor {
            0 => Align2::LEFT_TOP,
            1 => Align2::CENTER_TOP,
            2 => Align2::RIGHT_TOP,
            3 => Align2::LEFT_CENTER,
            4 => Align2::CENTER_CENTER,
            5 => Align2::RIGHT_CENTER,
            6 => Align2::LEFT_BOTTOM,
            7 => Align2::CENTER_BOTTOM,
            8 => Align2::RIGHT_BOTTOM,
            _ => Align2::CENTER_BOTTOM,
        };
        (align, vec2(self.offset.0, self.offset.1))
    }
}

impl SettingsAudio {
    pub fn device_id(&self) -> Option<SettingDeviceId> {
        self.device_id.clone()
    }
}

impl SettingsGeneral {
    pub fn log_to_file(&self) -> bool {
        self.log_to_file
    }

    pub fn level(&self) -> Level {
        Level::from(self.level)
    }
}

impl SettingsManager {
    pub fn new(path: &str) -> Self {
        let path = PathBuf::from(path);

        let settings = match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<SettingsApp>(&content) {
                Ok(settings) => settings,
                Err(e) => Self::recover_from_broken(&path, &content, e),
            },
            Err(_) => {
                let settings = SettingsApp::default();
                if let Ok(content) = toml::to_string_pretty(&settings) {
                    let _ = std::fs::write(&path, content);
                }
                settings
            }
        };

        Self { path, settings }
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
