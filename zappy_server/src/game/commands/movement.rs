//! Movement commands: `Forward`, `Right`, `Left` and `Eject`.
//!
//! Implements position and facing changes with toroidal wrap-around, emitting
//! the corresponding `ppo`/`pex` GUI events. `Eject` pushes every other player
//! sharing the tile one step in the ejector's facing and notifies each with the
//! direction they were pushed from.

use crate::game::world::World;
use crate::protocol::Command;
use crate::game::player::Direction;

/// Executes movement-related commands (Forward, Right, Left, Eject).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    match command {
        Command::Forward => {
            let player = world.players.get_mut(&player_id).expect("Player should exist");
            match player.direction {
                Direction::North => player.y = if player.y == 0 { world.height - 1 } else { player.y - 1 },
                Direction::East => player.x = (player.x + 1) % world.width,
                Direction::South => player.y = (player.y + 1) % world.height,
                Direction::West => player.x = if player.x == 0 { world.width - 1 } else { player.x - 1 },
            }
            let o = match player.direction { Direction::North => 1, Direction::East => 2, Direction::South => 3, Direction::West => 4 };
            let (x, y) = (player.x, player.y);
            world.gui_events.push_back(format!("ppo #{} {} {} {}\n", player_id, x, y, o));
            "ok\n".to_string()
        }
        Command::Right => {
            let player = world.players.get_mut(&player_id).expect("Player should exist");
            player.direction = player.direction.turn_right();
            let o = match player.direction { Direction::North => 1, Direction::East => 2, Direction::South => 3, Direction::West => 4 };
            let (x, y) = (player.x, player.y);
            world.gui_events.push_back(format!("ppo #{} {} {} {}\n", player_id, x, y, o));
            "ok\n".to_string()
        }
        Command::Left => {
            let player = world.players.get_mut(&player_id).expect("Player should exist");
            player.direction = player.direction.turn_left();
            let o = match player.direction { Direction::North => 1, Direction::East => 2, Direction::South => 3, Direction::West => 4 };
            let (x, y) = (player.x, player.y);
            world.gui_events.push_back(format!("ppo #{} {} {} {}\n", player_id, x, y, o));
            "ok\n".to_string()
        }
        Command::Eject => {
            let (px, py, pdir) = {
                let player = world.players.get(&player_id).expect("Player should exist");
                (player.x, player.y, player.direction)
            };

            let mut targets = Vec::new();
            for (&id, p) in world.players.iter() {
                if id != player_id && p.x == px && p.y == py {
                    targets.push(id);
                }
            }

            if targets.is_empty() {
                return "ko\n".to_string();
            }

            for target_id in targets {
                let (mut tx, mut ty) = (px as i32, py as i32);
                match pdir {
                    Direction::North => ty -= 1,
                    Direction::East => tx += 1,
                    Direction::South => ty += 1,
                    Direction::West => tx -= 1,
                }
                let final_x = tx.rem_euclid(world.width as i32) as u32;
                let final_y = ty.rem_euclid(world.height as i32) as u32;

                if let Some(target) = world.players.get_mut(&target_id) {
                    target.x = final_x;
                    target.y = final_y;
                    

                    let k = crate::game::commands::compute_direction(
                        final_x as i32, final_y as i32, target.direction,
                        px as i32, py as i32,
                        world.width as i32, world.height as i32
                    );
                    target.notifications.push_back(format!("eject {}\n", k));
                    
                    let o = match target.direction { Direction::North => 1, Direction::East => 2, Direction::South => 3, Direction::West => 4 };
                    world.gui_events.push_back(format!("ppo #{} {} {} {}\n", target_id, target.x, target.y, o));
                }
            }

            world.gui_events.push_back(format!("pex #{}\n", player_id));

            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}
