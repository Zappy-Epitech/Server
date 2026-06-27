use crate::game::world::{World, Resource};
use crate::game::player::Direction;
use crate::protocol::Command;

/// Executes resource interaction commands (Inventory, Take, Set, Look).
pub fn execute(command: Command, player_id: usize, world: &mut World) -> String {
    match command {
        Command::Inventory => {
            let player = world.players.get(&player_id).expect("Player should exist");
            let f = player.get_food_count(world.freq);
            let l = player.inventory.get(&Resource::Linemate).unwrap_or(&0);
            let d = player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
            let s = player.inventory.get(&Resource::Sibur).unwrap_or(&0);
            let m = player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
            let p = player.inventory.get(&Resource::Phiras).unwrap_or(&0);
            let t = player.inventory.get(&Resource::Thystame).unwrap_or(&0);

            format!("[ l {}, d {}, s {}, m {}, p {}, t {}, f {} ]\n", l, d, s, m, p, t, f)
        }
        Command::Look => {
            let (px, py, dir, level) = {
                let player = world.players.get(&player_id).expect("Player should exist");
                (player.x, player.y, player.direction, player.level)
            };

            let mut tiles_content = Vec::new();

            for d in 0..=level {
                for lateral in -(d as i32)..=(d as i32) {
                    let (mut tx, mut ty) = (px as i32, py as i32);
                    
                    match dir {
                        Direction::North => {
                            tx += lateral;
                            ty -= d as i32;
                        }
                        Direction::East => {
                            tx += d as i32;
                            ty += lateral;
                        }
                        Direction::South => {
                            tx -= lateral;
                            ty += d as i32;
                        }
                        Direction::West => {
                            tx -= d as i32;
                            ty -= lateral;
                        }
                    }
                    
                    let final_x = tx.rem_euclid(world.width as i32) as u32;
                    let final_y = ty.rem_euclid(world.height as i32) as u32;

                    tiles_content.push(world.get_tile_content(final_x, final_y));
                }
            }

            format!("[ {} ]\n", tiles_content.join(", "))
        }
        Command::Take(obj) => {
            let resource = if let Some(r) = Resource::from_str(&obj) { r } else { return "ko\n".to_string(); };

            let (px, py, success) = {
                let player = world.players.get(&player_id).expect("Player should exist");
                let (x, y) = (player.x, player.y);
                let tile = world.get_tile_mut(x, y);
                let count = tile.resources.get_mut(&resource);
                if let Some(c) = count {
                    if *c > 0 {
                        *c -= 1;
                        (x, y, true)
                    } else { (x, y, false) }
                } else { (x, y, false) }
            };

            if success {
                let resource_id = match resource {
                    Resource::Food => 0, Resource::Linemate => 1, Resource::Deraumere => 2,
                    Resource::Sibur => 3, Resource::Mendiane => 4, Resource::Phiras => 5, Resource::Thystame => 6,
                };
                world.gui_events.push_back(format!("pgt #{} {}\n", player_id, resource_id));

                if let Some(player) = world.players.get_mut(&player_id) {
                    if resource == Resource::Food {
                        player.add_food(world.freq);
                    } else {
                        *player.inventory.entry(resource).or_insert(0) += 1;
                    }
                    let f = player.get_food_count(world.freq);
                    let l = *player.inventory.get(&Resource::Linemate).unwrap_or(&0);
                    let d = *player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
                    let s = *player.inventory.get(&Resource::Sibur).unwrap_or(&0);
                    let m = *player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
                    let p = *player.inventory.get(&Resource::Phiras).unwrap_or(&0);
                    let t = *player.inventory.get(&Resource::Thystame).unwrap_or(&0);
                    player.last_food_count = f;
                    world.gui_events.push_back(format!("pin #{} {} {} {} {} {} {} {} {} {}\n", player_id, px, py, f, l, d, s, m, p, t));
                }
                world.gui_events.push_back(crate::gui::commands::map::format_bct(world, px, py));
                "ok\n".to_string()
            } else {
                "ko\n".to_string()
            }
        }
        Command::Set(obj) => {
            let resource = if let Some(r) = Resource::from_str(&obj) { r } else { return "ko\n".to_string(); };

            let (px, py, has_resource) = {
                if let Some(player) = world.players.get_mut(&player_id) {
                    let has = if resource == Resource::Food {
                        player.get_food_count(world.freq) > 0
                    } else {
                        *player.inventory.get(&resource).unwrap_or(&0) > 0
                    };
                    (player.x, player.y, has)
                } else {
                    return "ko\n".to_string();
                }
            };

            if has_resource {
                let resource_id = match resource {
                    Resource::Food => 0, Resource::Linemate => 1, Resource::Deraumere => 2,
                    Resource::Sibur => 3, Resource::Mendiane => 4, Resource::Phiras => 5, Resource::Thystame => 6,
                };
                world.gui_events.push_back(format!("pdr #{} {}\n", player_id, resource_id));

                if let Some(player) = world.players.get_mut(&player_id) {
                    if resource == Resource::Food {
                        player.remove_food(world.freq);
                    } else if let Some(count) = player.inventory.get_mut(&resource) {
                        if *count > 0 { *count -= 1; }
                    }
                    let f = player.get_food_count(world.freq);
                    let l = *player.inventory.get(&Resource::Linemate).unwrap_or(&0);
                    let d = *player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
                    let s = *player.inventory.get(&Resource::Sibur).unwrap_or(&0);
                    let m = *player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
                    let p = *player.inventory.get(&Resource::Phiras).unwrap_or(&0);
                    let t = *player.inventory.get(&Resource::Thystame).unwrap_or(&0);
                    player.last_food_count = f;
                    world.gui_events.push_back(format!("pin #{} {} {} {} {} {} {} {} {} {}\n", player_id, px, py, f, l, d, s, m, p, t));
                }

                {
                    let tile = world.get_tile_mut(px, py);
                    *tile.resources.entry(resource).or_insert(0) += 1;
                }
                
                world.gui_events.push_back(crate::gui::commands::map::format_bct(world, px, py));
                "ok\n".to_string()
            } else {
                "ko\n".to_string()
            }
        }
        Command::Fork => {
            let (tx, ty, team_name) = {
                let player = world.players.get(&player_id).expect("Player should exist");
                (player.x, player.y, player.team.clone())
            };
            
            let egg_id = world.next_egg_id;
            world.next_egg_id += 1;

            let life_duration = std::time::Duration::from_secs_f64(1260.0 / world.freq as f64);
            world.eggs.push(crate::game::world::Egg {
                id: egg_id,
                x: tx,
                y: ty,
                team: team_name,
                death_time: std::time::Instant::now() + life_duration,
            });

            world.gui_events.push_back(format!("enw #{} #{} {} {}\n", egg_id, player_id, tx, ty));
            
            "ok\n".to_string()
        }
        _ => "ko\n".to_string(),
    }
}
