#[derive(PartialEq, Eq)]
pub enum StreamAction {
    Reconnect {
        transcribed: bool,
    },
    Stop,
}
