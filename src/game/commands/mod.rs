pub mod movement;
pub mod interaction;
pub mod social;
pub mod incantation;

use crate::game::world::World;
use crate::protocol::Command;

/// Dispatches a command to the appropriate module and returns its responses.
/// 
/// According to RFC-ZAPPY-001, some commands have an immediate response 
/// (like Incantation's "Elevation underway\n") and all commands have a 
/// final response after their time delay.
/// 
/// Returns a tuple: `(Option<ImmediateResponse>, FinalResponse)`.
pub fn execute(command: Command, player_id: usize, world: &mut World) -> (Option<String>, String) {
    match command {
        Command::Forward | Command::Right | Command::Left => {
            (None, movement::execute(command, player_id, world))
        }
        Command::Inventory | Command::Take(_) | Command::Set(_) => {
            (None, interaction::execute(command, player_id, world))
        }
        Command::Broadcast(_) => {
            (None, social::execute(command, player_id, world))
        }
        Command::Incantation => {
            incantation::handle_start(player_id, world)
        }
        Command::ConnectNbr => {
            let team_name = {
                let player = world.players.get(&player_id).expect("Player should exist");
                player.team.clone()
            };
            let slots = world.team_slots.get(&team_name).unwrap_or(&0);
            (None, format!("{}\n", slots))
        }
        _ => (None, "ok\n".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_nbr_dispatch() {
        let mut world = World::new(10, 10, vec!["Team1".to_string()], 5);
        let player_id = world.add_player("Team1", 100).unwrap();
        
        let (immediate, final_res) = execute(Command::ConnectNbr, player_id, &mut world);
        
        assert!(immediate.is_none());
        assert_eq!(final_res, "4\n"); // 5 - 1
    }

    #[test]
    fn test_incantation_immediate_dispatch() {
        let mut world = World::new(10, 10, vec!["Team1".to_string()], 5);
        let player_id = world.add_player("Team1", 100).unwrap();
        
        let (x, y) = {
            let player = world.players.get(&player_id).unwrap();
            (player.x, player.y)
        };
        world.get_tile_mut(x, y).resources.insert(crate::game::world::Resource::Linemate, 1);
        
        let (immediate, _) = execute(Command::Incantation, player_id, &mut world);
        
        assert_eq!(immediate, Some("Elevation underway\n".to_string()));
    }
}
