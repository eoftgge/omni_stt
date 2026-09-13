#[derive(serde::Deserialize, Default)]
pub struct VoskText {
    #[serde(default)]
    text: String,
}

#[derive(serde::Deserialize, Default)]
pub struct VoskPartial {
    #[serde(default)]
    partial: String,
}
