/// Represents the various commands sent by a graphical client (GUI).
#[derive(Debug, Clone)]
pub enum GuiCommand {
    /// Request the map size (`msz`).
    MapSize,
    /// Request the content of a specific tile (`bct X Y`).
    TileContent(u32, u32),
    /// Request the content of the entire map (`mct`).
    MapContent,
    /// Request the names of all teams (`tna`).
    TeamNames,
    /// Request the position and orientation of a specific player (`ppo #n`).
    PlayerPosition(usize),
    /// Request the level of a specific player (`plv #n`).
    PlayerLevel(usize),
    /// Request the inventory of a specific player (`pin #n`).
    PlayerInventory(usize),
    /// Request the current time unit frequency (`sgt`).
    TimeRequest,
    /// Request a modification of the time unit frequency (`sst T`).
    TimeUpdate(u32),
}

impl GuiCommand {
    /// Parses a string into a GuiCommand enum.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gui_commands() {
        assert!(matches!(GuiCommand::from_str("msz"), Some(GuiCommand::MapSize)));
        assert!(matches!(GuiCommand::from_str("mct"), Some(GuiCommand::MapContent)));
        assert!(matches!(GuiCommand::from_str("tna"), Some(GuiCommand::TeamNames)));
        assert!(matches!(GuiCommand::from_str("sgt"), Some(GuiCommand::TimeRequest)));

        if let Some(GuiCommand::TileContent(x, y)) = GuiCommand::from_str("bct 5 10") {
            assert_eq!(x, 5);
            assert_eq!(y, 10);
        } else { panic!("Failed to parse bct"); }

        if let Some(GuiCommand::PlayerPosition(id)) = GuiCommand::from_str("ppo #42") {
            assert_eq!(id, 42);
        } else { panic!("Failed to parse ppo"); }

        if let Some(GuiCommand::PlayerLevel(id)) = GuiCommand::from_str("plv #42") {
            assert_eq!(id, 42);
        } else { panic!("Failed to parse plv"); }

        if let Some(GuiCommand::PlayerInventory(id)) = GuiCommand::from_str("pin #42") {
            assert_eq!(id, 42);
        } else { panic!("Failed to parse pin"); }

        if let Some(GuiCommand::TimeUpdate(t)) = GuiCommand::from_str("sst 150") {
            assert_eq!(t, 150);
        } else { panic!("Failed to parse sst"); }

        // Invalid commands
        assert!(GuiCommand::from_str("unknown").is_none());
        assert!(GuiCommand::from_str("bct 5").is_none()); // Missing Y
        assert!(GuiCommand::from_str("ppo #abc").is_none()); // Not a number
    }
}
