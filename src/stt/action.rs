pub enum StreamAction {
    Reconnect {
        transcribed: bool,
    },
    Stop,
}
