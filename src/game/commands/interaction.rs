use crate::game::world::{World, Resource};
use crate::protocol::Command;

/// Executes resource interaction commands (Inventory, Take, Set).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    match command {
        Command::Inventory => {
            let player = world.players.get(&player_id).expect("Player should exist");
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
            let resource = if let Some(r) = Resource::from_str(&obj) {
                r
            } else {
                return "ko\n".to_string();
            };

            let player = world.players.get(&player_id).expect("Player should exist");
            let (px, py) = (player.x, player.y);
            
            let tile = world.get_tile_mut(px, py);
            let count = tile.resources.get_mut(&resource);
            
            if let Some(c) = count {
                if *c > 0 {
                    *c -= 1;
                    let player = world.players.get_mut(&player_id).unwrap();
                    if resource == Resource::Food {
                        player.add_food(world.freq);
                    } else {
                        *player.inventory.entry(resource).or_insert(0) += 1;
                    }
                    "ok\n".to_string()
                } else {
                    "ko\n".to_string()
                }
            } else {
                "ko\n".to_string()
            }
        }
        Command::Set(_obj) => {
            // TODO: Transfer obj from player to tile
            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}
