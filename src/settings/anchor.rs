use eframe::egui::Align2;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Anchor {
    LeftTop = 0,
    CenterTop = 1,
    RightTop = 2,
    LeftCenter = 3,
    Center = 4,
    RightCenter = 5,
    LeftBottom = 6,
    #[default]
    CenterBottom = 7,
    RightBottom = 8,
}

impl Anchor {
    pub const ALL: [Self; 9] = [
        Self::LeftTop,
        Self::CenterTop,
        Self::RightTop,
        Self::LeftCenter,
        Self::Center,
        Self::RightCenter,
        Self::LeftBottom,
        Self::CenterBottom,
        Self::RightBottom,
    ];

    pub fn align(self) -> Align2 {
        match self {
            Self::LeftTop => Align2::LEFT_TOP,
            Self::CenterTop => Align2::CENTER_TOP,
            Self::RightTop => Align2::RIGHT_TOP,
            Self::LeftCenter => Align2::LEFT_CENTER,
            Self::Center => Align2::CENTER_CENTER,
            Self::RightCenter => Align2::RIGHT_CENTER,
            Self::LeftBottom => Align2::LEFT_BOTTOM,
            Self::CenterBottom => Align2::CENTER_BOTTOM,
            Self::RightBottom => Align2::RIGHT_BOTTOM,
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::LeftTop => "↖",
            Self::CenterTop => "↑",
            Self::RightTop => "↗",
            Self::LeftCenter => "←",
            Self::Center => "•",
            Self::RightCenter => "→",
            Self::LeftBottom => "↙",
            Self::CenterBottom => "↓",
            Self::RightBottom => "↘",
        }
    }

    pub fn default_offset(self) -> (f32, f32) {
        const PAD: f32 = 30.0;

        let x = match self {
            Self::LeftTop | Self::LeftCenter | Self::LeftBottom => PAD,
            Self::RightTop | Self::RightCenter | Self::RightBottom => -PAD,
            _ => 0.0,
        };
        let y = match self {
            Self::LeftTop | Self::CenterTop | Self::RightTop => PAD,
            Self::LeftBottom | Self::CenterBottom | Self::RightBottom => -PAD,
            _ => 0.0,
        };

        (x, y)
    }

    fn name(self) -> &'static str {
        match self {
            Self::LeftTop => "left_top",
            Self::CenterTop => "center_top",
            Self::RightTop => "right_top",
            Self::LeftCenter => "left_center",
            Self::Center => "center_center",
            Self::RightCenter => "right_center",
            Self::LeftBottom => "left_bottom",
            Self::CenterBottom => "center_bottom",
            Self::RightBottom => "right_bottom",
        }
    }
}

impl Serialize for Anchor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for Anchor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Stored {
            Name(String),
            Index(u8),
        }

        match Stored::deserialize(deserializer)? {
            Stored::Name(name) => Self::ALL
                .into_iter()
                .find(|anchor| anchor.name() == name)
                .ok_or_else(|| D::Error::custom(format!("unknown anchor: {name}"))),

            // Configs written before this was an enum hold a bare 0..8 index.
            // Accepting them keeps every other setting alive: a hard failure
            // here trips `recover_from_broken` and resets the whole file.
            Stored::Index(index) => Ok(Self::ALL
                .get(index as usize)
                .copied()
                .unwrap_or_default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use super::Anchor;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Wrapper {
        anchor: Anchor,
    }

    #[test]
    fn every_anchor_survives_a_round_trip() {
        for anchor in Anchor::ALL {
            let encoded = toml::to_string(&Wrapper { anchor }).unwrap();
            let decoded: Wrapper = toml::from_str(&encoded).unwrap();
            assert_eq!(decoded.anchor, anchor, "сломался round-trip для {encoded}");
        }
    }

    #[test]
    fn configs_from_before_the_enum_still_load() {
        // A plain index is what every omni.toml in the wild holds today. Losing
        // this would reset the whole settings file, not just the anchor.
        for (index, expected) in Anchor::ALL.into_iter().enumerate() {
            let decoded: Wrapper = toml::from_str(&format!("anchor = {index}")).unwrap();
            assert_eq!(decoded.anchor, expected, "индекс {index}");
        }
    }

    #[test]
    fn a_nonsense_index_falls_back_instead_of_failing() {
        let decoded: Wrapper = toml::from_str("anchor = 99").unwrap();
        assert_eq!(decoded.anchor, Anchor::default());
    }
}
