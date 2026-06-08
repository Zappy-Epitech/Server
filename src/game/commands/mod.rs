pub mod movement;
pub mod interaction;
pub mod social;
pub mod incantation;

use crate::game::world::World;
use crate::protocol::Command;

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
            let slots = world.team_slots.get(&world.players.get(&player_id).unwrap().team).unwrap_or(&0);
            (None, format!("{}\n", slots))
        }
        _ => (None, "ok\n".to_string()),
    }
}
