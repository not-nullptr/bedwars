#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Status,
    Login,
    Configuration,
    Play,
}
