use crate::game::world::World;
use crate::protocol::Command;
use crate::game::player::Direction;

/// Executes movement-related commands (Forward, Right, Left).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    let player = world.players.get_mut(&player_id).expect("Player should exist");

    match command {
        Command::Forward => {
            match player.direction {
                Direction::North => player.y = if player.y == 0 { world.height - 1 } else { player.y - 1 },
                Direction::East => player.x = (player.x + 1) % world.width,
                Direction::South => player.y = (player.y + 1) % world.height,
                Direction::West => player.x = if player.x == 0 { world.width - 1 } else { player.x - 1 },
            }
            "ok\n".to_string()
        }
        Command::Right => {
            player.direction = player.direction.turn_right();
            "ok\n".to_string()
        }
        Command::Left => {
            player.direction = player.direction.turn_left();
            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}

