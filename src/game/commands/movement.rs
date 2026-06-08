use crate::game::world::World;
use crate::protocol::Command;

/// Executes movement-related commands (Forward, Right, Left).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    match command {
        Command::Forward => {
            // TODO: Move player forward (respect world wrap-around)
            "ok\n".to_string()
        }
        Command::Right => {
            // TODO: Rotate player right
            "ok\n".to_string()
        }
        Command::Left => {
            // TODO: Rotate player left
            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}

