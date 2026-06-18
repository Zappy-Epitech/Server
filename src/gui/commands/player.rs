use crate::game::world::{World, Resource};
use crate::game::player::Direction;
use crate::protocol::gui::GuiCommand;

pub fn execute(command: GuiCommand, world: &World) -> Vec<String> {
    let mut responses = Vec::new();

    match command {
        GuiCommand::PlayerPosition(id) => {
            if let Some(player) = world.players.get(&id) {
                let orientation = match player.direction {
                    Direction::North => 1,
                    Direction::East => 2,
                    Direction::South => 3,
                    Direction::West => 4,
                };
                responses.push(format!("ppo {} {} {} {}\n", id, player.x, player.y, orientation));
            } else {
                responses.push("sbp\n".to_string());
            }
        }
        GuiCommand::PlayerLevel(id) => {
            if let Some(player) = world.players.get(&id) {
                responses.push(format!("plv {} {}\n", id, player.level));
            } else {
                responses.push("sbp\n".to_string());
            }
        }
        GuiCommand::PlayerInventory(id) => {
            if let Some(player) = world.players.get(&id) {
                let f = player.get_food_count(world.freq);
                let l = player.inventory.get(&Resource::Linemate).unwrap_or(&0);
                let d = player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
                let s = player.inventory.get(&Resource::Sibur).unwrap_or(&0);
                let m = player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
                let p = player.inventory.get(&Resource::Phiras).unwrap_or(&0);
                let t = player.inventory.get(&Resource::Thystame).unwrap_or(&0);
                
                responses.push(format!("pin {} {} {} {} {} {} {} {} {} {}\n", 
                    id, player.x, player.y, f, l, d, s, m, p, t));
            } else {
                responses.push("sbp\n".to_string());
            }
        }
        _ => {}
    }

    responses
}
