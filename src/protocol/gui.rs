#[derive(Debug, Clone)]
pub enum GuiCommand {
    MapSize,
    TileContent(u32, u32),
    MapContent,
    TeamNames,
    PlayerPosition(usize),
    PlayerLevel(usize),
    PlayerInventory(usize),
    TimeRequest,
    TimeUpdate(u32),
}

impl GuiCommand {
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() { return None; }

        match parts[0] {
            "msz" => Some(GuiCommand::MapSize),
            "bct" if parts.len() == 3 => {
                let x = parts[1].parse().ok()?;
                let y = parts[2].parse().ok()?;
                Some(GuiCommand::TileContent(x, y))
            }
            "mct" => Some(GuiCommand::MapContent),
            "tna" => Some(GuiCommand::TeamNames),
            "ppo" if parts.len() == 2 => {
                let id = parts[1].trim_start_matches('#').parse().ok()?;
                Some(GuiCommand::PlayerPosition(id))
            }
            "plv" if parts.len() == 2 => {
                let id = parts[1].trim_start_matches('#').parse().ok()?;
                Some(GuiCommand::PlayerLevel(id))
            }
            "pin" if parts.len() == 2 => {
                let id = parts[1].trim_start_matches('#').parse().ok()?;
                Some(GuiCommand::PlayerInventory(id))
            }
            "sgt" => Some(GuiCommand::TimeRequest),
            "sst" if parts.len() == 2 => {
                let t = parts[1].parse().ok()?;
                Some(GuiCommand::TimeUpdate(t))
            }
            _ => None,
        }
    }
}
