use std::collections::HashMap;
use crate::game::world::Resource;

/// The four cardinal directions a player can face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// Returns the direction after a 90-degree right turn.
    pub fn turn_right(self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Returns the direction after a 90-degree left turn.
    pub fn turn_left(self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
        }
    }
}

/// Represents an inhabitant of Trantor (a player).
pub struct Player {
    /// Unique identifier for the player.
    pub id: usize,
    /// Current X position on the map.
    pub x: u32,
    /// Current Y position on the map.
    pub y: u32,
    /// Current direction the player is facing.
    pub direction: Direction,
    /// Current elevation level of the player (starts at 1).
    pub level: u32,
    /// Player's inventory of resources.
    pub inventory: HashMap<Resource, u32>,
    /// Time units remaining before death (starts at 1260).
    pub life_units: u32,
    /// The name of the team the player belongs to.
    pub team: String,
}

impl Player {
    /// Creates a new player at the given coordinates for a specific team.
    /// 
    /// Initial inventory contains 10 units of food.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_initialization() {
        let player = Player::new(1, 5, 5, Direction::North, "Team1".to_string());
        
        assert_eq!(player.id, 1);
        assert_eq!(player.x, 5);
        assert_eq!(player.y, 5);
        assert_eq!(player.direction, Direction::North);
        assert_eq!(player.level, 1);
        assert_eq!(player.inventory.get(&Resource::Food), Some(&10));
        assert_eq!(player.life_units, 1260);
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
