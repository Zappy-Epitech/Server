use crate::game::world::World;
use crate::config::ServerConfig;
use crate::protocol::gui::GuiCommand;
use std::time::{Duration, Instant};

/// Executes time-related GUI commands (`sgt`, `sst`).
/// 
/// Dynamically scales the remaining time of all ongoing events (death, commands, respawns)
/// when the server frequency is modified.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_commands() {
        let mut config = ServerConfig {
            port: 4242,
            width: 10,
            height: 10,
            teams: vec!["TeamA".to_string()],
            clients_nb: 5,
            freq: 100,
            bonus: false,
        };
        let mut world = World::new(config.width, config.height, config.teams.clone(), config.clients_nb, config.freq);

        let res_sgt = execute(GuiCommand::TimeRequest, &mut world, &mut config);
        assert_eq!(res_sgt.len(), 1);
        assert_eq!(res_sgt[0], "sgt 100\n");

        let res_sst = execute(GuiCommand::TimeUpdate(200), &mut world, &mut config);
        assert_eq!(res_sst.len(), 1);
        assert_eq!(res_sst[0], "sst 200\n");
        assert_eq!(config.freq, 200);
        assert_eq!(world.freq, 200);
    }
}
