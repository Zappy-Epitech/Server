//! The game world: the toroidal Trantor map and all its state.
//!
//! Holds the [`World`] aggregate — the grid of [`Tile`]s (indexed `x + y*width`
//! with wrap-around), the live [`Player`]s, the pending [`Egg`]s, per-team
//! spawn slots and the queue of GUI events to broadcast. It also owns the
//! [`Resource`] kinds with their spawn densities, periodic resource respawning,
//! player add/remove, tile access helpers and the team victory check.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use rand::Rng;
use crate::game::player::{Player, Direction};

/// A resource (food and the six elevation stones) that can lie on a tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    Food,
    Linemate,
    Deraumere,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
}

impl Resource {
    /// Returns a slice of every resource variant, in canonical order.
    pub fn all() -> &'static [Resource] {
        &[
            Resource::Food,
            Resource::Linemate,
            Resource::Deraumere,
            Resource::Sibur,
            Resource::Mendiane,
            Resource::Phiras,
            Resource::Thystame,
        ]
    }

    /// Parses a resource from its lowercase protocol name, or `None` if unknown.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "food" => Some(Resource::Food),
            "linemate" => Some(Resource::Linemate),
            "deraumere" => Some(Resource::Deraumere),
            "sibur" => Some(Resource::Sibur),
            "mendiane" => Some(Resource::Mendiane),
            "phiras" => Some(Resource::Phiras),
            "thystame" => Some(Resource::Thystame),
            _ => None,
        }
    }

    /// Returns the lowercase protocol name of this resource.
    pub fn to_str(self) -> &'static str {
        match self {
            Resource::Food => "food",
            Resource::Linemate => "linemate",
            Resource::Deraumere => "deraumere",
            Resource::Sibur => "sibur",
            Resource::Mendiane => "mendiane",
            Resource::Phiras => "phiras",
            Resource::Thystame => "thystame",
        }
    }

    /// Target density of this resource per tile, used to size the global pool
    /// kept alive on the map by [`World::spawn_resources`].
    pub fn density(self) -> f64 {
        match self {
            Resource::Food => 0.5,
            Resource::Linemate => 0.3,
            Resource::Deraumere => 0.15,
            Resource::Sibur => 0.1,
            Resource::Mendiane => 0.1,
            Resource::Phiras => 0.08,
            Resource::Thystame => 0.05,
        }
    }
}

/// A single map cell, holding the quantity of each resource present on it.
#[derive(Debug, Clone)]
pub struct Tile {
    /// Count of each resource currently lying on this tile.
    pub resources: HashMap<Resource, u32>,
}

impl Tile {
    /// Creates an empty tile with no resources.
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

/// An egg laid by a `Fork`; hatches into a new player slot until it expires.
pub struct Egg {
    /// Unique egg identifier (used in `enw`/`ebo`/`edi` GUI events).
    pub id: usize,
    /// X position of the egg on the map.
    pub x: u32,
    /// Y position of the egg on the map.
    pub y: u32,
    /// Team the egg belongs to; only that team can hatch from it.
    pub team: String,
    /// Instant at which the egg expires if not yet hatched.
    pub death_time: Instant,
}

/// The complete authoritative game state for one running server.
pub struct World {
    /// Map width in tiles.
    pub width: u32,
    /// Map height in tiles.
    pub height: u32,
    /// Flat row-major tile grid, indexed `x + y * width` with toroidal wrap.
    pub tiles: Vec<Tile>,
    /// All connected players, keyed by player id.
    pub players: HashMap<usize, Player>,
    /// Remaining initial connection slots per team name.
    pub team_slots: HashMap<String, usize>,
    /// Live (unhatched) eggs on the map.
    pub eggs: Vec<Egg>,
    /// Current time-unit frequency; higher means faster game time.
    pub freq: u32,
    /// Instant at which resources are next replenished.
    pub next_spawn_time: Instant,
    /// Queue of GUI protocol lines awaiting broadcast to graphic clients.
    pub gui_events: std::collections::VecDeque<String>,
    /// Monotonic counter for assigning the next player id.
    next_player_id: usize,
    /// Monotonic counter for assigning the next egg id.
    pub next_egg_id: usize,
}

impl World {
    /// Builds a new world of `width`x`height`, seeds team slots from
    /// `clients_per_team`, schedules the first resource respawn (scaled by
    /// `freq`) and spawns the initial resource pool.
    pub fn new(width: u32, height: u32, teams: Vec<String>, clients_per_team: usize, freq: u32) -> Self {
        let mut team_slots = HashMap::new();
        for team in teams {
            team_slots.insert(team, clients_per_team);
        }

        let now = Instant::now();
        let spawn_interval = Duration::from_secs_f64(20.0 / freq as f64);

        let mut world = Self {
            width,
            height,
            tiles: vec![Tile::new(); (width * height) as usize],
            players: HashMap::new(),
            team_slots,
            eggs: Vec::new(),
            freq,
            next_spawn_time: now + spawn_interval,
            gui_events: std::collections::VecDeque::new(),
            next_player_id: 1,
            next_egg_id: 1,
        };
        world.spawn_resources();
        world
    }

    /// Adds a player to `team_name`, returning its new id, or `None` if the team
    /// is full. The player hatches at an existing egg of the team if one exists
    /// (consuming it), otherwise it consumes a free team slot and spawns at a
    /// random position with a random facing.
    pub fn add_player(&mut self, team_name: &str, freq: u32) -> Option<usize> {
        let mut spawn_pos = None;

        if let Some(pos) = self.eggs.iter().position(|e| e.team == team_name) {
            let egg = self.eggs.remove(pos);
            self.gui_events.push_back(format!("ebo #{}\n", egg.id));
            spawn_pos = Some((egg.x, egg.y));
        } else {
            let slots = self.team_slots.get_mut(team_name)?;
            if *slots > 0 {
                *slots -= 1;
                let mut rng = rand::thread_rng();
                spawn_pos = Some((rng.gen_range(0..self.width), rng.gen_range(0..self.height)));
            }
        }

        let (x, y) = spawn_pos?;
        let mut rng = rand::thread_rng();
        let id = self.next_player_id;
        self.next_player_id += 1;

        let direction = match rng.gen_range(0..4) {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            _ => Direction::West,
        };

        let player = Player::new(id, x, y, direction, team_name.to_string(), freq);
        self.players.insert(id, player);
        Some(id)
    }

    /// Removes a player and returns its connection slot to its team's pool.
    pub fn remove_player(&mut self, player_id: usize) {
        if let Some(player) = self.players.remove(&player_id) {
            if let Some(slots) = self.team_slots.get_mut(&player.team) {
                *slots += 1;
            }
        }
    }

    /// Returns a shared reference to the tile at `(x, y)`.
    pub fn get_tile(&self, x: u32, y: u32) -> &Tile {
        let idx = (y * self.width + x) as usize;
        &self.tiles[idx]
    }

    /// Returns a mutable reference to the tile at `(x, y)`.
    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> &mut Tile {
        let idx = (y * self.width + x) as usize;
        &mut self.tiles[idx]
    }

    /// Returns the space-separated `Look`-style contents of a tile: one
    /// `player` token per player standing on it, then one token per resource
    /// unit present.
    pub fn get_tile_content(&self, x: u32, y: u32) -> String {
        let mut content = Vec::new();
        
        for player in self.players.values() {
            if player.x == x && player.y == y {
                content.push("player".to_string());
            }
        }

        let tile = self.get_tile(x, y);
        for resource in Resource::all() {
            if let Some(&count) = tile.resources.get(resource) {
                for _ in 0..count {
                    content.push(resource.to_str().to_string());
                }
            }
        }

        content.join(" ")
    }

    /// Returns the winning team's name if any team has at least six players who
    /// have reached the maximum level (8), otherwise `None`.
    pub fn check_victory(&self) -> Option<String> {
        let mut team_counts = HashMap::new();
        for player in self.players.values() {
            if player.level >= 8 {
                *team_counts.entry(player.team.clone()).or_insert(0) += 1;
            }
        }
        for (team, count) in team_counts {
            if count >= 6 {
                return Some(team);
            }
        }
        None
    }

    /// Tops up the map so that each resource's total count reaches its
    /// density-derived target, scattering any shortfall onto random tiles.
    pub fn spawn_resources(&mut self) {
        let mut rng = rand::thread_rng();
        let total_tiles = (self.width * self.height) as f64;

        for &resource in Resource::all() {
            let total_quantity = (total_tiles * resource.density()).ceil() as u32;
            
            let mut current_quantity = 0;
            for tile in &self.tiles {
                current_quantity += tile.resources.get(&resource).unwrap_or(&0);
            }

            if current_quantity < total_quantity {
                for _ in 0..(total_quantity - current_quantity) {
                    let x = rng.gen_range(0..self.width);
                    let y = rng.gen_range(0..self.height);
                    let tile = self.get_tile_mut(x, y);
                    *tile.resources.entry(resource).or_insert(0) += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let teams = vec!["Team1".to_string(), "Team2".to_string()];
        let world = World::new(10, 10, teams, 5, 100);

        assert_eq!(world.width, 10);
        assert_eq!(world.height, 10);
        assert_eq!(world.tiles.len(), 100);
        assert_eq!(world.team_slots.get("Team1"), Some(&5));
    }

    #[test]
    fn test_resource_spawning() {
        let world = World::new(10, 10, vec!["Team1".to_string()], 2, 100);
        
        let mut total_food = 0;
        for tile in &world.tiles {
            total_food += tile.resources.get(&Resource::Food).unwrap_or(&0);
        }
        
        assert!(total_food >= 50); // 100 * 0.5
    }

    #[test]
    fn test_add_remove_player() {
        let mut world = World::new(10, 10, vec!["Team1".to_string()], 1, 100);
        
        let id = world.add_player("Team1", 100).expect("Should add player");
        assert_eq!(world.team_slots.get("Team1"), Some(&0));
        assert!(world.add_player("Team1", 100).is_none());

        world.remove_player(id);
        assert_eq!(world.team_slots.get("Team1"), Some(&1));
    }
}
