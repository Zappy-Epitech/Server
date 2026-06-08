use std::time::Instant;

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

pub struct PendingCommand {
    pub command: Command,
    pub end_time: Instant,
}
