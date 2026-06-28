//! Map-related GUI commands: `msz`, `bct`, `mct`, `tna`.
//!
//! Answers graphic-client queries about the world layout — map size, single or
//! full tile contents, and team names — and exposes [`format_bct`], the shared
//! helper that serializes a tile's resources into a `bct` line (also reused
//! across the engine whenever a tile changes).

use crate::game::world::{World, Resource};
use crate::config::ServerConfig;
use crate::protocol::gui::GuiCommand;

/// Executes map-related GUI commands (`msz`, `bct`, `mct`, `tna`).
pub fn execute(command: GuiCommand, world: &World, config: &ServerConfig) -> Vec<String> {
    let mut responses = Vec::new();

    match command {
        GuiCommand::MapSize => {
            responses.push(format!("msz {} {}\n", config.width, config.height));
        }
        GuiCommand::TileContent(x, y) => {
            if x < config.width && y < config.height {
                responses.push(format_bct(world, x, y));
            } else {
                responses.push("sbp\n".to_string());
            }
        }
        GuiCommand::MapContent => {
            for y in 0..config.height {
                for x in 0..config.width {
                    responses.push(format_bct(world, x, y));
                }
            }
        }
        GuiCommand::TeamNames => {
            for team in &config.teams {
                responses.push(format!("tna {}\n", team));
            }
        }
        _ => {}
    }

    responses
}

/// Formats the content of a specific tile into a `bct` protocol string.
pub fn format_bct(world: &World, x: u32, y: u32) -> String {
    let tile = world.get_tile(x, y);
    let f = tile.resources.get(&Resource::Food).unwrap_or(&0);
    let l = tile.resources.get(&Resource::Linemate).unwrap_or(&0);
    let d = tile.resources.get(&Resource::Deraumere).unwrap_or(&0);
    let s = tile.resources.get(&Resource::Sibur).unwrap_or(&0);
    let m = tile.resources.get(&Resource::Mendiane).unwrap_or(&0);
    let p = tile.resources.get(&Resource::Phiras).unwrap_or(&0);
    let t = tile.resources.get(&Resource::Thystame).unwrap_or(&0);
    format!("bct {} {} {} {} {} {} {} {} {}\n", x, y, f, l, d, s, m, p, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::world::World;

    fn setup_world_config() -> (World, ServerConfig) {
        let config = ServerConfig {
            port: 4242,
            width: 5,
            height: 5,
            teams: vec!["TeamA".to_string(), "TeamB".to_string()],
            clients_nb: 2,
            freq: 100,
            bonus: false,
        };
        let world = World::new(config.width, config.height, config.teams.clone(), config.clients_nb, config.freq);
        (world, config)
    }

    #[test]
    fn test_map_size_command() {
        let (world, config) = setup_world_config();
        let res = execute(GuiCommand::MapSize, &world, &config);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], "msz 5 5\n");
    }

    #[test]
    fn test_team_names_command() {
        let (world, config) = setup_world_config();
        let res = execute(GuiCommand::TeamNames, &world, &config);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], "tna TeamA\n");
        assert_eq!(res[1], "tna TeamB\n");
    }

    #[test]
    fn test_tile_content_command() {
        let (world, config) = setup_world_config();
        let res = execute(GuiCommand::TileContent(0, 0), &world, &config);
        assert_eq!(res.len(), 1);
        assert!(res[0].starts_with("bct 0 0"));
        
        let res_out_of_bounds = execute(GuiCommand::TileContent(10, 10), &world, &config);
        assert_eq!(res_out_of_bounds.len(), 1);
        assert_eq!(res_out_of_bounds[0], "sbp\n");
    }
}
