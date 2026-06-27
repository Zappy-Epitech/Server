pub mod app;

pub enum ServerEvent {
    Log(String),
    ClientConnected,
    ClientDisconnected,
    PlayerJoinedTeam(String),
    PlayerDied(String),
    FreqChanged(u32),
    GameOver(String),
    MapSnapshot(Vec<(u32, u32)>),
}
