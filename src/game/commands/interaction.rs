use crate::game::world::{World, Resource};
use crate::protocol::Command;

/// Executes resource interaction commands (Inventory, Take, Set).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    let player = world.players.get_mut(&player_id).expect("Player should exist");

    match command {
        Command::Inventory => {
            let f = player.get_food_count(world.freq);
            let l = player.inventory.get(&Resource::Linemate).unwrap_or(&0);
            let d = player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
            let s = player.inventory.get(&Resource::Sibur).unwrap_or(&0);
            let m = player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
            let p = player.inventory.get(&Resource::Phiras).unwrap_or(&0);
            let t = player.inventory.get(&Resource::Thystame).unwrap_or(&0);

            format!("[ l {}, d {}, s {}, m {}, p {}, t {}, f {} ]\n", l, d, s, m, p, t, f)
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
