use crate::game::world::{World, Resource};
use crate::game::player::Direction;
use crate::protocol::gui::GuiCommand;

/// Executes player-related GUI commands (`ppo`, `plv`, `pin`).
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
                responses.push(format!("ppo #{} {} {} {}\n", id, player.x, player.y, orientation));
            } else {
                responses.push("sbp\n".to_string());
            }
        }
        GuiCommand::PlayerLevel(id) => {
            if let Some(player) = world.players.get(&id) {
                responses.push(format!("plv #{} {}\n", id, player.level));
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
                
                responses.push(format!("pin #{} {} {} {} {} {} {} {} {} {}\n", 
                    id, player.x, player.y, f, l, d, s, m, p, t));
            } else {
                responses.push("sbp\n".to_string());
            }
        }
        _ => {}
    }

    responses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_info_commands() {
        let mut world = World::new(10, 10, vec!["TeamA".to_string()], 5, 100);
        let player_id = world.add_player("TeamA", 100).unwrap();
        
        // ppo
        let res_ppo = execute(GuiCommand::PlayerPosition(player_id), &world);
        assert_eq!(res_ppo.len(), 1);
        assert!(res_ppo[0].starts_with(&format!("ppo #{}", player_id)));

        // plv
        let res_plv = execute(GuiCommand::PlayerLevel(player_id), &world);
        assert_eq!(res_plv.len(), 1);
        assert_eq!(res_plv[0], format!("plv #{} 1\n", player_id));

        // pin
        let res_pin = execute(GuiCommand::PlayerInventory(player_id), &world);
        assert_eq!(res_pin.len(), 1);
        assert!(res_pin[0].starts_with(&format!("pin #{}", player_id)));

        // Invalid player
        let res_invalid = execute(GuiCommand::PlayerLevel(999), &world);
        assert_eq!(res_invalid[0], "sbp\n");
    }
}
