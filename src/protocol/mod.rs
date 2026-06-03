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
}
