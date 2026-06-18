pub mod map;
pub mod player;
pub mod time;

use crate::game::world::World;
use crate::config::ServerConfig;
use crate::protocol::gui::GuiCommand;

/// Dispatches GUI commands to their respective handlers.
/// 
/// Returns a list of strings representing the responses to be sent back to the GUI client.
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
    }
}
