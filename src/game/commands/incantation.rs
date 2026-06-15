use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::LazyLock;
use crate::game::world::{World, Tile, Resource};

// Elevation requirements
struct Req {
    nb_players: u32,
    linemate: u32,
    deraumere: u32,
    sibur: u32,
    mendiane: u32,
    phiras: u32,
    thystame: u32,
}

// Returns the requirements for the ritual based on the player's current level
fn get_requirements(level: u32) -> Option<Req> {
    match level {
        1 => Some(Req { nb_players: 1, linemate: 1, deraumere: 0, sibur: 0, mendiane: 0, phiras: 0, thystame: 0 }),
        2 => Some(Req { nb_players: 2, linemate: 1, deraumere: 1, sibur: 1, mendiane: 0, phiras: 0, thystame: 0 }),
        3 => Some(Req { nb_players: 2, linemate: 2, deraumere: 0, sibur: 1, mendiane: 0, phiras: 2, thystame: 0 }),
        4 => Some(Req { nb_players: 4, linemate: 1, deraumere: 1, sibur: 2, mendiane: 0, phiras: 1, thystame: 0 }),
        5 => Some(Req { nb_players: 4, linemate: 1, deraumere: 2, sibur: 1, mendiane: 3, phiras: 0, thystame: 0 }),
        6 => Some(Req { nb_players: 6, linemate: 1, deraumere: 2, sibur: 3, mendiane: 0, phiras: 1, thystame: 0 }),
        7 => Some(Req { nb_players: 6, linemate: 2, deraumere: 2, sibur: 2, mendiane: 2, phiras: 2, thystame: 1 }),
        _ => None,
    }
}

// Checks if the tile contains enough of each resource stone
fn check_resources(tile: &Tile, req: &Req) -> bool {
    let get_count = |res: Resource| -> u32 {
        *tile.resources.get(&res).unwrap_or(&0)
    };

    get_count(Resource::Linemate) >= req.linemate &&
    get_count(Resource::Deraumere) >= req.deraumere &&
    get_count(Resource::Sibur) >= req.sibur &&
    get_count(Resource::Mendiane) >= req.mendiane &&
    get_count(Resource::Phiras) >= req.phiras &&
    get_count(Resource::Thystame) >= req.thystame
}

// Decrements the required amount of stones from the tile
fn consume_resources(tile: &mut Tile, req: &Req) {
    let mut consume = |res: Resource, qty: u32| {
        if let Some(count) = tile.resources.get_mut(&res) {
            if *count >= qty {
                *count -= qty;
            } else {
                *count = 0;
            }
        }
    };

    consume(Resource::Linemate, req.linemate);
    consume(Resource::Deraumere, req.deraumere);
    consume(Resource::Sibur, req.sibur);
    consume(Resource::Mendiane, req.mendiane);
    consume(Resource::Phiras, req.phiras);
    consume(Resource::Thystame, req.thystame);
}

// Counts the number of players of a specific level standing on a specific tile
fn count_players_on_tile(world: &World, x: u32, y: u32, level: u32) -> u32 {
    let mut count = 0;
    for player in world.players.values() {
        if player.x == x && player.y == y && player.level == level {
            count += 1;
        }
    }
    count
}

// Map containing every active rituals
// HashMap key: player_id (the player initiating the incantation).
// HashMap value:
// - None if the incantation failed at the start.
// - Some(level) if it started successfully, where level is the player's level at start.
static ACTIVE_INCANTATIONS: LazyLock<Mutex<HashMap<usize, Option<u32>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

/// Handles the initiation of an incantation ritual (RFC Section 7.12).
/// 
/// Returns `Some("Elevation underway\n")` or `Some("ko\n")`.
pub fn handle_start(player_id: usize, world: &mut World) -> Option<String> {
    let mut active = ACTIVE_INCANTATIONS.lock().unwrap();

    let (x, y, level) = match world.players.get(&player_id) {
        Some(p) => (p.x, p.y, p.level),
        None => return Some("ko\n".to_string()),
    };

    let req = match get_requirements(level) {
        Some(r) => r,
        None => return Some("ko\n".to_string()),
    };

    let tile = world.get_tile(x, y);
    let resource_check = check_resources(tile, &req);
    let player_count = count_players_on_tile(world, x, y, level);

    if resource_check && player_count >= req.nb_players {
        active.insert(player_id, Some(level));
        Some("Elevation underway\n".to_string())
    } else {
        active.insert(player_id, None);
        Some("ko\n".to_string())
    }
}

/// Executes the final logic of an incantation ritual after the time delay.
pub fn execute(player_id: usize, world: &mut World) -> String {
    let level_at_start = {
        let mut active = ACTIVE_INCANTATIONS.lock().unwrap();
        active.remove(&player_id)
    };

    let start_level = match level_at_start {
        Some(Some(l)) => l,
        Some(None) => return String::new(),
        None => world.players.get(&player_id).map(|p| p.level).unwrap_or(1),
    };

    let (x, y, level) = match world.players.get(&player_id) {
        Some(p) => (p.x, p.y, p.level),
        None => return "ko\n".to_string(),
    };

    if level > start_level {
        return format!("Current level: {}\n", level);
    }

    let req = match get_requirements(start_level) {
        Some(r) => r,
        None => return "ko\n".to_string(),
    };

    let tile = world.get_tile(x, y);
    let resource_check = check_resources(tile, &req);
    let player_count = count_players_on_tile(world, x, y, start_level);

    if resource_check && player_count >= req.nb_players {
        let tile_mut = world.get_tile_mut(x, y);
        consume_resources(tile_mut, &req);

        let new_level = start_level + 1;
        for p in world.players.values_mut() {
            if p.x == x && p.y == y && p.level == start_level {
                p.level = new_level;
            }
        }

        format!("Current level: {}\n", new_level)
    } else {
        "ko\n".to_string()
    }
}
