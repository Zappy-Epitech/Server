use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use crate::game::world::Resource;
use crate::protocol::PendingCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn turn_right(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub fn turn_left(self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }
}

pub struct Player {
    pub id: usize,
    pub x: u32,
    pub y: u32,
    pub direction: Direction,
    pub level: u32,
    pub inventory: HashMap<Resource, u32>,
    pub team: String,
    pub commands: VecDeque<PendingCommand>,
    pub last_command_end: Instant,
    pub death_time: Instant,
}

impl Player {
    pub fn new(id: usize, x: u32, y: u32, direction: Direction, team: String, freq: u32) -> Self {
        let mut inventory = HashMap::new();
        inventory.insert(Resource::Food, 10);

        let now = Instant::now();
        let life_duration = Duration::from_secs_f64(1260.0 / freq as f64);

        Self {
            id,
            x,
            y,
            direction,
            level: 1,
            inventory,
            team,
            commands: VecDeque::new(),
            last_command_end: now,
            death_time: now + life_duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_initialization() {
        let player = Player::new(1, 5, 5, Direction::North, "Team1".to_string(), 100);
        
        assert_eq!(player.id, 1);
        assert_eq!(player.x, 5);
        assert_eq!(player.y, 5);
        assert_eq!(player.direction, Direction::North);
        assert_eq!(player.level, 1);
        assert_eq!(player.inventory.get(&Resource::Food), Some(&10));
    }

    #[test]
    fn test_direction_turns() {
        let mut dir = Direction::North;
        
        dir = dir.turn_right();
        assert_eq!(dir, Direction::East);
        
        dir = dir.turn_right();
        assert_eq!(dir, Direction::South);
        
        dir = dir.turn_left();
        assert_eq!(dir, Direction::East);
        
        dir = dir.turn_left();
        assert_eq!(dir, Direction::North);
    }
}
