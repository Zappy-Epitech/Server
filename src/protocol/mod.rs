use std::time::Instant;

/// The various commands an AI client can send to the server.
#[derive(Debug, Clone)]
pub enum Command {
    Forward,
    Right,
    Left,
    Look,
    Inventory,
    Broadcast(String),
    ConnectNbr,
    Fork,
    Eject,
    Take(String),
    Set(String),
    Incantation,
}

impl Command {
    /// Returns the base duration of the command in game time units.
    pub fn duration(&self) -> u32 {
        match self {
            Command::Forward | Command::Right | Command::Left | Command::Look | 
            Command::Broadcast(_) | Command::Eject | Command::Take(_) | Command::Set(_) => 7,
            Command::Inventory => 1,
            Command::Fork => 42,
            Command::Incantation => 300,
            Command::ConnectNbr => 0,
        }
    }

    /// Parses a string into a Command enum.
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() { return None; }

        match parts[0] {
            "Forward" => Some(Command::Forward),
            "Right" => Some(Command::Right),
            "Left" => Some(Command::Left),
            "Look" => Some(Command::Look),
            "Inventory" => Some(Command::Inventory),
            "Connect_nbr" => Some(Command::ConnectNbr),
            "Fork" => Some(Command::Fork),
            "Eject" => Some(Command::Eject),
            "Take" if parts.len() > 1 => Some(Command::Take(parts[1].to_string())),
            "Set" if parts.len() > 1 => Some(Command::Set(parts[1].to_string())),
            "Incantation" => Some(Command::Incantation),
            "Broadcast" if parts.len() > 1 => Some(Command::Broadcast(parts[1..].join(" "))),
            _ => None,
        }
    }
}

/// A command that has been received and scheduled for future execution.
pub struct PendingCommand {
    /// The command to be executed.
    pub command: Command,
    /// The precise moment when the command execution should complete.
    pub end_time: Instant,
}
