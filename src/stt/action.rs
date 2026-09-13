pub enum StreamAction {
    Reconnect { transcribed: bool },
    Idle,
    Stop,
}
