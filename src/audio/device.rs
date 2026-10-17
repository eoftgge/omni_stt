#[cfg(test)]
mod tests;

use crate::errors::OmniSttErrors;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, DeviceId, StreamConfig};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// One entry of the user's selection, as it is stored in `omni.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioSource {
    pub kind: DeviceKind,
    pub id: SettingDeviceId,
}

impl AudioSource {
    /// False for a source saved on another system; see [`SettingDeviceId`].
    pub fn is_native(&self) -> bool {
        matches!(self.id, SettingDeviceId::Native(_))
    }
}

/// Which side of a device the audio is taken from.
///
/// Both are opened with `build_input_stream` — an output device is captured
/// through a loopback, an input device is an ordinary microphone. The kind
/// decides only two things: which config the device is asked for, and which
/// of the host's two lists it came from.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    #[default]
    Output,
    Input,
}

impl DeviceKind {
    pub const ALL: [Self; 2] = [Self::Output, Self::Input];

    pub fn label(self) -> &'static str {
        match self {
            Self::Output => "System audio",
            Self::Input => "Microphone",
        }
    }
}

pub struct MappableAvailableDevices(cpal::Host, Vec<AvailableDevice>);

#[derive(Clone)]
pub struct AvailableDevice {
    inner: Device,
    name: String,
    id: SettingDeviceId,
    kind: DeviceKind,
}

/// A device id as `omni.toml` stores it.
///
/// The text names the audio host (`wasapi:…`, `coreaudio:…`), and cpal only
/// knows the hosts of the system it was built for. A config carried on a USB
/// stick between Windows and a Mac holds ids from both. Ids from the other
/// system are kept as text: they never match a device here, and are written
/// back untouched, so they work again once the stick is back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingDeviceId {
    Native(DeviceId),
    Foreign(String),
}

impl AvailableDevice {
    pub fn new(device: Device, kind: DeviceKind) -> Option<Self> {
        let id = device.id().ok()?;
        let desc = device.description().ok()?;
        Some(Self {
            inner: device,
            id: SettingDeviceId::Native(id),
            name: desc.name().into(),
            kind,
        })
    }

    pub fn from_host(host: &cpal::Host, kind: DeviceKind) -> Option<Self> {
        let device = match kind {
            DeviceKind::Output => host.default_output_device()?,
            DeviceKind::Input => host.default_input_device()?,
        };
        Self::new(device, kind)
    }

    /// The capture configuration for this device.
    ///
    /// Asking the wrong side fails — `default_input_config` errors on a pair
    /// of speakers, `default_output_config` on a microphone — so the kind has
    /// to choose. This is the only place in the pipeline where the difference
    /// exists; everything downstream sees one `StreamConfig`.
    pub fn stream_config(&self) -> Result<StreamConfig, OmniSttErrors> {
        let supported = match self.kind {
            DeviceKind::Output => self.inner.default_output_config()?,
            DeviceKind::Input => self.inner.default_input_config()?,
        };
        Ok(supported.config())
    }

    pub fn into_inner(self) -> Device {
        self.inner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> &SettingDeviceId {
        &self.id
    }

    pub fn kind(&self) -> DeviceKind {
        self.kind
    }
}

impl MappableAvailableDevices {
    pub fn from_host(host: cpal::Host) -> Self {
        let devices = Self::enumerate(&host);
        Self(host, devices)
    }

    pub fn refresh(&mut self) {
        self.1 = Self::enumerate(&self.0);
    }

    pub fn from_default_host() -> Self {
        let host = cpal::default_host();
        Self::from_host(host)
    }

    /// Keyed by the pair, never by the id alone.
    ///
    /// `HostTrait::input_devices` and `output_devices` filter the same device
    /// list by `supports_input` / `supports_output`, so a headset lands in
    /// both — with the same `DeviceId`, because the id belongs to the device
    /// and not to a direction. Matching on the id alone would silently return
    /// whichever side happened to be enumerated first.
    pub fn get(&self, kind: DeviceKind, id: &SettingDeviceId) -> Option<&AvailableDevice> {
        self.1.iter().find(|d| d.kind() == kind && d.id() == id)
    }

    /// Turns the saved selection into devices to open.
    ///
    /// An empty selection means the default output device: that keeps the
    /// behaviour of every version before this one, and gives a fresh install
    /// something to listen to without any setup at all. So does a selection
    /// made entirely on another system.
    ///
    /// An entry whose device has since disappeared is reported and skipped
    /// rather than failing the start — one unplugged microphone must not take
    /// the other sources down with it.
    pub fn resolve(&self, sources: &[AudioSource]) -> Vec<AvailableDevice> {
        if !sources.iter().any(AudioSource::is_native) {
            return AvailableDevice::from_host(&self.0, DeviceKind::Output)
                .into_iter()
                .collect();
        }

        sources
            .iter()
            .filter(|source| source.is_native())
            .filter_map(|source| {
                let found = self.get(source.kind, &source.id).cloned();
                if found.is_none() {
                    tracing::warn!("A saved {:?} source is gone, skipping it", source.kind);
                }
                found
            })
            .collect()
    }

    pub fn iter(&self, kind: DeviceKind) -> impl Iterator<Item = &AvailableDevice> {
        self.1.iter().filter(move |d| d.kind() == kind)
    }

    fn enumerate(host: &cpal::Host) -> Vec<AvailableDevice> {
        let outputs = host
            .output_devices()
            .into_iter()
            .flatten()
            .filter_map(|device| AvailableDevice::new(device, DeviceKind::Output));

        let inputs = host
            .input_devices()
            .into_iter()
            .flatten()
            .filter_map(|device| AvailableDevice::new(device, DeviceKind::Input));

        outputs.chain(inputs).collect()
    }
}

impl SettingDeviceId {
    pub fn new(id: DeviceId) -> Self {
        Self::Native(id)
    }

    /// `None` for an id from another system.
    pub fn inner(&self) -> Option<&DeviceId> {
        match self {
            Self::Native(id) => Some(id),
            Self::Foreign(_) => None,
        }
    }
}

impl serde::Serialize for SettingDeviceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Native(id) => id.to_string().serialize(serializer),
            Self::Foreign(text) => text.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for SettingDeviceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Never an error: one id from another system used to reset the whole
        // config to defaults, API key included.
        let text = String::deserialize(deserializer)?;
        Ok(match DeviceId::from_str(&text) {
            Ok(id) => Self::Native(id),
            Err(e) => {
                tracing::debug!("Keeping a device id from another system: {e}");
                Self::Foreign(text)
            }
        })
    }
}
