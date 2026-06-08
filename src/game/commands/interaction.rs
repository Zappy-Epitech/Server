use crate::game::world::World;
use crate::protocol::Command;

/// Executes resource interaction commands (Inventory, Take, Set).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    match command {
        Command::Inventory => {
            // TODO: Format inventory string [food n, linemate n, ...]
            "[food 10]\n".to_string()
        }
        Command::Take(obj) => {
            // TODO: Transfer obj from tile to player
            "ok\n".to_string()
        }
        Command::Set(obj) => {
            // TODO: Transfer obj from player to tile
            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}
