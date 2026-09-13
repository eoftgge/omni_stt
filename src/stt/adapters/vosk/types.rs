#[derive(serde::Deserialize, Default)]
pub struct VoskText {
    #[serde(default)]
    pub text: String,
}

#[derive(serde::Deserialize, Default)]
pub struct VoskPartial {
    #[serde(default)]
    pub partial: String,
}
