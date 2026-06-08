use crate::game::world::World;

pub fn handle_start(_player_id: usize, _world: &mut World) -> (Option<String>, String) {
    (Some("Elevation underway\n".to_string()), "Current level: 2\n".to_string())
}

pub fn execute(_player_id: usize, _world: &mut World) -> String {
    "Current level: 2\n".to_string()
}
