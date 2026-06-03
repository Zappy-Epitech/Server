use std::collections::HashMap;
use crate::game::world::Resource;

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
    pub life_units: u32,
    pub team: String,
}

impl Player {
    pub fn new(id: usize, x: u32, y: u32, direction: Direction, team: String) -> Self {
        let mut inventory = HashMap::new();
        inventory.insert(Resource::Food, 10);

        Self {
            id,
            x,
            y,
            direction,
            level: 1,
            inventory,
            life_units: 1260,
            team,
        }
    }
}
