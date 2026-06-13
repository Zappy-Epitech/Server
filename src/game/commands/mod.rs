pub mod movement;
pub mod interaction;
pub mod social;
pub mod incantation;

use crate::game::world::World;
use crate::protocol::Command;

/// Handles the immediate part of a command (RFC requirement).
/// Only returns a message if the command requires an immediate response.
pub fn init(command: &Command, player_id: usize, world: &mut World) -> Option<String> {
    match command {
        Command::Incantation => incantation::handle_start(player_id, world),
        _ => None,
    }
}

/// Handles the final execution of a command after its time delay.
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    match command {
        Command::Forward | Command::Right | Command::Left => {
            movement::execute(command, player_id, world)
        }
        Command::Inventory | Command::Take(_) | Command::Set(_) => {
            interaction::execute(command, player_id, world)
        }
        Command::Broadcast(_) => {
            social::execute(command, player_id, world)
        }
        Command::Incantation => {
            incantation::execute(player_id, world)
        }
        Command::ConnectNbr => {
            let team_name = {
                let player = world.players.get(&player_id).expect("Player should exist");
                player.team.clone()
            };
            let slots = world.team_slots.get(&team_name).unwrap_or(&0);
            format!("{}\n", slots)
        }
        _ => "ok\n".to_string(),
    }
}
