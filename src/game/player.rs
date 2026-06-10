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

    pub fn add_food(&mut self, freq: u32) {
        let duration = Duration::from_secs_f64(126.0 / freq as f64);
        self.death_time += duration;
    }

    pub fn remove_food(&mut self, freq: u32) {
        let duration = Duration::from_secs_f64(126.0 / freq as f64);
        self.death_time -= duration;
    }

    pub fn get_food_count(&self, freq: u32) -> u32 {
        let now = Instant::now();
        if now >= self.death_time { return 0; }
        
        let remaining = self.death_time.duration_since(now).as_secs_f64();
        let unit_duration = 126.0 / freq as f64;
        
        (remaining / unit_duration).ceil() as u32
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
    }

    #[test]
    fn test_food_count() {
        let mut player = Player::new(1, 0, 0, Direction::North, "T".to_string(), 100);
        assert_eq!(player.get_food_count(100), 10);
        
        player.add_food(100);
        assert_eq!(player.get_food_count(100), 11);
    }
}
