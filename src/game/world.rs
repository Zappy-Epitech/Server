use std::collections::HashMap;
use std::time::{Duration, Instant};
use rand::Rng;
use crate::game::player::{Player, Direction};

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

#[derive(Debug, Clone)]
pub struct Tile {
    pub resources: HashMap<Resource, u32>,
}

impl Tile {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }
}

pub struct Egg {
    pub x: u32,
    pub y: u32,
    pub team: String,
}

pub struct World {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub players: HashMap<usize, Player>,
    pub team_slots: HashMap<String, usize>,
    pub eggs: Vec<Egg>,
    pub freq: u32,
    pub next_spawn_time: Instant,
    next_player_id: usize,
}

impl World {
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
            next_player_id: 1,
        };
        world.spawn_resources();
        world
    }

    pub fn add_player(&mut self, team_name: &str, freq: u32) -> Option<usize> {
        let mut spawn_pos = None;

        if let Some(pos) = self.eggs.iter().position(|e| e.team == team_name) {
            let egg = self.eggs.remove(pos);
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

    pub fn remove_player(&mut self, player_id: usize) {
        if let Some(player) = self.players.remove(&player_id) {
            if let Some(slots) = self.team_slots.get_mut(&player.team) {
                *slots += 1;
            }
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> &Tile {
        let idx = (y * self.width + x) as usize;
        &self.tiles[idx]
    }

    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> &mut Tile {
        let idx = (y * self.width + x) as usize;
        &mut self.tiles[idx]
    }

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
