#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct SubtitleBlock {
    pub(crate) speaker: Option<String>,
    pub(crate) text: String,
}

impl SubtitleBlock {
    pub fn new(speaker: Option<String>) -> Self {
        Self {
            speaker,
            ..Default::default()
        }
    }
}
