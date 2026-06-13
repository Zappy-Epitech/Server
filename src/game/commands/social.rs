use crate::game::world::World;
use crate::game::player::Direction;
use crate::protocol::Command;

/// Executes social and communication commands (Broadcast).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    let text = match command {
        Command::Broadcast(t) => t,
        _ => return "ko\n".to_string(),
    };

    let (sender_x, sender_y) = {
        let sender = world.players.get(&player_id).expect("Sender exists");
        (sender.x, sender.y)
    };

    let mut targets = Vec::new();
    for (&id, player) in world.players.iter() {
        if id != player_id {
            targets.push((id, player.x, player.y, player.direction));
        }
    }

    for (target_id, target_x, target_y, target_dir) in targets {
        let k = crate::game::commands::compute_direction(
            target_x as i32, target_y as i32, target_dir,
            sender_x as i32, sender_y as i32,
            world.width as i32, world.height as i32
        );
        
        let msg = format!("message {}, {}\n", k, text);
        if let Some(player) = world.players.get_mut(&target_id) {
            player.notifications.push_back(msg);
        }
    }

    "ok\n".to_string()
}
