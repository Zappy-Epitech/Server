pub mod map;
pub mod player;
pub mod time;

use crate::game::world::World;
use crate::config::ServerConfig;
use crate::protocol::gui::GuiCommand;

pub fn execute(command: GuiCommand, world: &mut World, config: &mut ServerConfig) -> Vec<String> {
    match command {
        GuiCommand::MapSize | GuiCommand::TileContent(_, _) | GuiCommand::MapContent | GuiCommand::TeamNames => {
            map::execute(command, world, config)
        }
        GuiCommand::PlayerPosition(_) | GuiCommand::PlayerLevel(_) | GuiCommand::PlayerInventory(_) => {
            player::execute(command, world)
        }
        GuiCommand::TimeRequest | GuiCommand::TimeUpdate(_) => {
            time::execute(command, world, config)
        }
        _ => vec!["suc\n".to_string()],
    }
}
