use crate::game::world::World;
use crate::config::ServerConfig;
use crate::protocol::gui::GuiCommand;
use std::time::{Duration, Instant};

pub fn execute(command: GuiCommand, world: &mut World, config: &mut ServerConfig) -> Vec<String> {
    match command {
        GuiCommand::TimeRequest => {
            vec![format!("sgt {}\n", config.freq)]
        }
        GuiCommand::TimeUpdate(new_freq) => {
            if new_freq > 0 && new_freq != config.freq {
                let old_freq = config.freq as f64;
                let new_freq_f64 = new_freq as f64;
                let ratio = old_freq / new_freq_f64;
                let now = Instant::now();

                for player in world.players.values_mut() {
                    if player.death_time > now {
                        let remaining = player.death_time.duration_since(now).as_secs_f64();
                        player.death_time = now + Duration::from_secs_f64(remaining * ratio);
                    }
                    if player.last_command_end > now {
                        let remaining = player.last_command_end.duration_since(now).as_secs_f64();
                        player.last_command_end = now + Duration::from_secs_f64(remaining * ratio);
                    }
                    for cmd in player.commands.iter_mut() {
                        if cmd.end_time > now {
                            let remaining = cmd.end_time.duration_since(now).as_secs_f64();
                            cmd.end_time = now + Duration::from_secs_f64(remaining * ratio);
                        }
                    }
                }

                if world.next_spawn_time > now {
                    let remaining = world.next_spawn_time.duration_since(now).as_secs_f64();
                    world.next_spawn_time = now + Duration::from_secs_f64(remaining * ratio);
                }

                world.freq = new_freq;
                config.freq = new_freq;
            }
            vec![format!("sst {}\n", config.freq)]
        }
        _ => Vec::new(),
    }
}
