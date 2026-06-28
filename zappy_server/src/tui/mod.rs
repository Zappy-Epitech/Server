//! Bonus terminal dashboard (`--bonus`).
//!
//! Defines [`ServerEvent`], the message type the server thread sends over an
//! `mpsc` channel to the [`app`] module's Ratatui UI, decoupling the simulation
//! from rendering. Each variant carries just enough state for the dashboard to
//! update its counters, logs and live minimap.

pub mod app;

/// A snapshot event pushed from the server thread to the TUI for display.
pub enum ServerEvent {
    /// A log line to append to the live event stream.
    Log(String),
    /// A new client (AI or GUI) connected.
    ClientConnected,
    /// A client disconnected.
    ClientDisconnected,
    /// A player joined the named team.
    PlayerJoinedTeam(String),
    /// A player of the named team left or starved to death.
    PlayerDied(String),
    /// The server's time-unit frequency changed to the given value.
    FreqChanged(u32),
    /// The game ended; carries the winning team's name.
    GameOver(String),
    /// Current `(x, y)` positions of all players, for the live minimap.
    MapSnapshot(Vec<(u32, u32)>),
}
