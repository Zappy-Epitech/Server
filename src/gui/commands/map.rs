use crate::game::world::{World, Resource};
use crate::config::ServerConfig;
use crate::protocol::gui::GuiCommand;

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
