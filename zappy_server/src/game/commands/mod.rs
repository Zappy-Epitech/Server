pub mod movement;
pub mod interaction;
pub mod social;
pub mod incantation;

use crate::game::world::World;
use crate::protocol::Command;
use crate::game::player::Direction;

/// Handles the immediate part of a command (RFC requirement).
/// Only returns a message if the command requires an immediate response.
pub fn init(command: &Command, player_id: usize, world: &mut World) -> Option<String> {
    match command {
        Command::Incantation => incantation::handle_start(player_id, world),
        Command::Fork => {
            world.gui_events.push_back(format!("pfk #{}\n", player_id));
            None
        }
        _ => None,
    }
}

/// Handles the final execution of a command after its time delay.
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    match command {
        Command::Forward | Command::Right | Command::Left | Command::Eject => {
            movement::execute(command, player_id, world)
        }
        Command::Look | Command::Inventory | Command::Take(_) | Command::Set(_) | Command::Fork => {
            interaction::execute(command, player_id, world)
        }
        Command::Broadcast(_) => {
            social::execute(command, player_id, world)
        }
        Command::Incantation => {
            incantation::execute(player_id, world)
        }
        Command::ConnectNbr => {
            if let Some(player) = world.players.get(&player_id) {
                let team_name = player.team.clone();
                let initial_slots = *world.team_slots.get(&team_name).unwrap_or(&0);
                let egg_slots = world.eggs.iter().filter(|e| e.team == team_name).count();
                format!("{}\n", initial_slots + egg_slots)
            } else {
                "0\n".to_string()
            }
        }
    }
}

/// Calculates the relative direction (1-8) of a sound source on a toroidal map.
/// 
/// This implementation uses the **Minimum Image Convention** algorithm.
/// 
/// ### The Algorithm: Minimum Image Convention
/// In a periodic (toroidal) world, a sound source has an infinite number of "images" 
/// due to the map wrapping around. This algorithm ensures we always calculate the 
/// path to the *closest* image of the sender.
/// 
/// 1. **Shortest Vector**: We calculate the raw distance `dx` and `dy`. If a distance 
///    is greater than half the world size, we "wrap" it by subtracting/adding the 
///    full world size. This gives us the shortest possible vector `(dx, dy)` on a torus.
/// 
/// 2. **Trigonometry**: We use `atan2(dx, -dy)` to convert this vector into a 
///    geographic angle where 0° is North (upward in our grid).
/// 
/// 3. **Compass Mapping**: The angle is mapped to the RFC's 1-8 compass (8 slices of 45°).
/// 
/// 4. **Receiver Relativity**: Finally, we adjust the absolute direction by the 
///    receiver's current orientation (`rdir`) so that '1' always represents the 
///    tile directly in front of them.
pub fn compute_direction(rx: i32, ry: i32, rdir: Direction, sx: i32, sy: i32, w: i32, h: i32) -> u32 {
    if rx == sx && ry == sy { return 0; }

    let mut dx = sx - rx;
    let mut dy = sy - ry;

    if dx.abs() > w / 2 { dx -= dx.signum() * w; }
    if dy.abs() > h / 2 { dy -= dy.signum() * h; }

    let angle = (dx as f64).atan2(-dy as f64).to_degrees();
    let normalized_angle = if angle < 0.0 { angle + 360.0 } else { angle };

    let abs_k = (((normalized_angle + 22.5) % 360.0) / 45.0).floor() as u32 + 1;

    let dir_offset = match rdir {
        Direction::North => 0,
        Direction::East => 2,
        Direction::South => 4,
        Direction::West => 6,
    };

    let relative_k = if abs_k == 0 { 0 } else {
        let mut k = abs_k as i32 - dir_offset;
        while k <= 0 { k += 8; }
        k as u32
    };

    relative_k
}
