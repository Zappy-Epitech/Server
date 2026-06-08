use std::collections::HashMap;
use rand::Rng;
use crate::game::player::{Player, Direction};

/// The different types of resources available on Trantor.
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
    /// Returns all types of resources.
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

    /// Returns the density for a given resource.
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

/// A single tile on the world map.
#[derive(Debug, Clone)]
pub struct Tile {
    /// Resources present on this tile.
    pub resources: HashMap<Resource, u32>,
}

impl Tile {
    /// Creates a new empty tile.
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

/// The world of Trantor, a 2D grid with wrap-around logic.
pub struct World {
    /// Width of the map.
    pub width: u32,
    /// Height of the map.
    pub height: u32,
    /// The grid of tiles.
    pub tiles: Vec<Tile>,
    /// All players currently in the world, indexed by their ID.
    pub players: HashMap<usize, Player>,
    /// Number of available slots for each team.
    pub team_slots: HashMap<String, usize>,
    /// Counter for generating unique player IDs.
    next_player_id: usize,
}

impl World {
    /// Creates a new world with the given dimensions and populates it with resources.
    pub fn new(width: u32, height: u32, teams: Vec<String>, clients_per_team: usize) -> Self {
        let mut team_slots = HashMap::new();
        for team in teams {
            team_slots.insert(team, clients_per_team);
        }

        let mut world = Self {
            width,
            height,
            tiles: vec![Tile::new(); (width * height) as usize],
            players: HashMap::new(),
            team_slots,
            next_player_id: 1,
        };
        world.spawn_resources();
        world
    }

    /// Adds a new player to the world for the specified team if a slot is available.
    pub fn add_player(&mut self, team_name: &str) -> Option<usize> {
        let slots = self.team_slots.get_mut(team_name)?;
        if *slots == 0 {
            return None;
        }
        *slots -= 1;

        let mut rng = rand::thread_rng();
        let id = self.next_player_id;
        self.next_player_id += 1;

        let x = rng.gen_range(0..self.width);
        let y = rng.gen_range(0..self.height);
        let direction = match rng.gen_range(0..4) {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            _ => Direction::West,
        };

        let player = Player::new(id, x, y, direction, team_name.to_string());
        self.players.insert(id, player);
        Some(id)
    }

    /// Removes a player from the world and frees up a slot for their team.
    pub fn remove_player(&mut self, player_id: usize) {
        if let Some(player) = self.players.remove(&player_id) {
            if let Some(slots) = self.team_slots.get_mut(&player.team) {
                *slots += 1;
            }
        }
    }

    /// Returns a reference to a tile at the given coordinates.
    pub fn get_tile(&self, x: u32, y: u32) -> &Tile {
        let idx = (y * self.width + x) as usize;
        &self.tiles[idx]
    }

    /// Returns a mutable reference to a tile at the given coordinates.
    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> &mut Tile {
        let idx = (y * self.width + x) as usize;
        &mut self.tiles[idx]
    }

    /// Populates the world with resources according to their densities.
    pub fn spawn_resources(&mut self) {
        let mut rng = rand::thread_rng();
        let total_tiles = (self.width * self.height) as f64;

        for &resource in Resource::all() {
            let total_quantity = (total_tiles * resource.density()).ceil() as u32;
            for _ in 0..total_quantity {
                let x = rng.gen_range(0..self.width);
                let y = rng.gen_range(0..self.height);
                let tile = self.get_tile_mut(x, y);
                *tile.resources.entry(resource).or_insert(0) += 1;
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
        let world = World::new(10, 10, teams, 5);

        assert_eq!(world.width, 10);
        assert_eq!(world.height, 10);
        assert_eq!(world.tiles.len(), 100);
        assert_eq!(world.team_slots.get("Team1"), Some(&5));
    }

    #[test]
    fn test_resource_spawning() {
        let world = World::new(10, 10, vec!["Team1".to_string()], 2);
        
        let mut total_food = 0;
        for tile in &world.tiles {
            total_food += tile.resources.get(&Resource::Food).unwrap_or(&0);
        }
        
        assert!(total_food >= 50); // 100 * 0.5
    }

    #[test]
    fn test_add_remove_player() {
        let mut world = World::new(10, 10, vec!["Team1".to_string()], 1);
        
        let id = world.add_player("Team1").expect("Should add player");
        assert_eq!(world.team_slots.get("Team1"), Some(&0));
        assert!(world.add_player("Team1").is_none());

        world.remove_player(id);
        assert_eq!(world.team_slots.get("Team1"), Some(&1));
    }
}
