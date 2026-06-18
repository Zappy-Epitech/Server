pub mod map;

use crate::game::world::World;
use crate::config::ServerConfig;
use crate::protocol::gui::GuiCommand;

pub fn execute(command: GuiCommand, world: &World, config: &ServerConfig) -> Vec<String> {
    match command {
        GuiCommand::MapSize | GuiCommand::TileContent(_, _) | GuiCommand::MapContent | GuiCommand::TeamNames => {
            map::execute(command, world, config)
        }
        _ => vec!["suc\n".to_string()],
    }
}
