use crate::game::world::World;
use crate::protocol::Command;

pub fn execute(_command: Command, _player_id: usize, _world: &mut World) -> String {
    "ok\n".to_string()
}
