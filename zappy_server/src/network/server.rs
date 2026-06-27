use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use zappy_network::{NetworkServer, ServerEventHandler};

use crate::config::ServerConfig;
use crate::game::world::{Resource, World};
use crate::protocol::gui::GuiCommand;
use crate::protocol::{Command, PendingCommand};
use crate::tui::ServerEvent;

/// Represents the various states a client can be in.
#[derive(Debug, PartialEq)]
pub enum ClientState {
    /// Initial state, waiting for the client to send a team name.
    Authenticating,
    /// Client is an AI (drone) currently in the game.
    InGame(usize),
    /// Client is a graphical interface.
    Graphic,
}

pub struct Server {
    pub config: ServerConfig,
    pub world: World,
    pub tui_tx: Option<Sender<ServerEvent>>,
    pub client_states: HashMap<usize, ClientState>,
}

impl Server {
    pub fn new(config: ServerConfig, tui_tx: Option<Sender<ServerEvent>>) -> Self {
        let world = World::new(
            config.width,
            config.height,
            config.teams.clone(),
            config.clients_nb,
            config.freq,
        );

        Self {
            config,
            world,
            tui_tx,
            client_states: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let mut network = NetworkServer::new()?;
        let port = self.config.port;
        network.run(port, self)
    }

    fn log(&self, msg: String) {
        if let Some(tx) = &self.tui_tx {
            let _ = tx.send(ServerEvent::Log(msg));
        } else {
            println!("{}", msg);
        }
    }

    fn emit_event(&self, event: ServerEvent) {
        if let Some(tx) = &self.tui_tx {
            let _ = tx.send(event);
        }
    }

    pub fn broadcast_gui(&mut self, msg: &str, network: &mut NetworkServer) {
        for (client_id, state) in &self.client_states {
            if let ClientState::Graphic = state {
                network.send_message(*client_id, msg.as_bytes());
            }
        }
    }

    fn flush_notifications(&mut self, network: &mut NetworkServer) {
        for (client_id, state) in &self.client_states {
            if let ClientState::InGame(player_id) = state {
                if let Some(player) = self.world.players.get_mut(player_id) {
                    while let Some(notif) = player.notifications.pop_front() {
                        network.send_message(*client_id, notif.as_bytes());
                    }
                }
            }
        }
    }

    fn execute_command(&mut self, client_id: usize, cmd: Command, network: &mut NetworkServer) {
        let player_id = if let Some(ClientState::InGame(id)) = self.client_states.get(&client_id) {
            Some(*id)
        } else {
            None
        };

        if let Some(id) = player_id {
            let response = crate::game::commands::execute(cmd, id, &mut self.world);
            network.send_message(client_id, response.as_bytes());
        }
    }

    fn handle_gui_command(&mut self, client_id: usize, cmd: GuiCommand, network: &mut NetworkServer) {
        let responses = crate::gui::commands::execute(cmd.clone(), &mut self.world, &mut self.config);

        if let GuiCommand::TimeUpdate(_) = cmd {
            self.log(format!("GUI modified frequency to {}", self.config.freq));
            self.emit_event(ServerEvent::FreqChanged(self.config.freq));
        }

        for response in responses {
            network.send_message(client_id, response.as_bytes());
        }
    }
}

impl ServerEventHandler for Server {
    fn on_client_connected(&mut self, client_id: usize, network: &mut NetworkServer) {
        self.client_states.insert(client_id, ClientState::Authenticating);
        network.send_message(client_id, b"WELCOME\n");
        self.log(format!("New connection from client {}", client_id));
        self.emit_event(ServerEvent::ClientConnected);
    }

    fn on_client_disconnected(&mut self, client_id: usize, network: &mut NetworkServer) {
        self.log(format!("Client {} disconnected", client_id));
        self.emit_event(ServerEvent::ClientDisconnected);
        if let Some(state) = self.client_states.remove(&client_id) {
            if let ClientState::InGame(id) = state {
                self.broadcast_gui(&format!("pdi #{}\n", id), network);
                let team = self
                    .world
                    .players
                    .get(&id)
                    .map(|p| p.team.clone())
                    .unwrap_or_default();
                self.world.remove_player(id);
                self.emit_event(ServerEvent::PlayerDied(team));
            }
        }
    }

    fn on_message_received(&mut self, client_id: usize, msg: String, network: &mut NetworkServer) {
        let state = self.client_states.get(&client_id).map(|s| match s {
            ClientState::Authenticating => 0,
            ClientState::InGame(_) => 1,
            ClientState::Graphic => 2,
        });

        match state {
            Some(0) => {
                if msg == "GRAPHIC" {
                    self.client_states.insert(client_id, ClientState::Graphic);
                    network.send_message(client_id, b"smg Welcome to Zappy Server!\n");
                    network.send_message(client_id, format!("msz {} {}\n", self.config.width, self.config.height).as_bytes());
                    network.send_message(client_id, format!("sgt {}\n", self.config.freq).as_bytes());

                    for y in 0..self.config.height {
                        for x in 0..self.config.width {
                            network.send_message(client_id, crate::gui::commands::map::format_bct(&self.world, x, y).as_bytes());
                        }
                    }

                    for team in &self.config.teams {
                        network.send_message(client_id, format!("tna {}\n", team).as_bytes());
                    }

                    for player in self.world.players.values() {
                        let o = match player.direction {
                            crate::game::player::Direction::North => 1,
                            crate::game::player::Direction::East => 2,
                            crate::game::player::Direction::South => 3,
                            crate::game::player::Direction::West => 4,
                        };
                        network.send_message(client_id, format!("pnw #{} {} {} {} {} {}\n", player.id, player.x, player.y, o, player.level, player.team).as_bytes());

                        let f = player.get_food_count(self.world.freq);
                        let l = player.inventory.get(&Resource::Linemate).unwrap_or(&0);
                        let d = player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
                        let s = player.inventory.get(&Resource::Sibur).unwrap_or(&0);
                        let m = player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
                        let p = player.inventory.get(&Resource::Phiras).unwrap_or(&0);
                        let t = player.inventory.get(&Resource::Thystame).unwrap_or(&0);
                        network.send_message(client_id, format!("pin #{} {} {} {} {} {} {} {} {} {}\n", player.id, player.x, player.y, f, l, d, s, m, p, t).as_bytes());
                    }
                    for e in &self.world.eggs {
                        network.send_message(client_id, format!("enw #{} #0 {} {}\n", e.id, e.x, e.y).as_bytes());
                    }
                    self.log(format!("Graphic client connected (ID: {})", client_id));
                } else if let Some(player_id) = self.world.add_player(&msg, self.config.freq) {
                    self.client_states.insert(client_id, ClientState::InGame(player_id));
                    let initial_slots = *self.world.team_slots.get(&msg).unwrap_or(&0);
                    let egg_slots = self.world.eggs.iter().filter(|e| e.team == msg).count();
                    let response = format!("{}\n{} {}\n", initial_slots + egg_slots, self.config.width, self.config.height);
                    network.send_message(client_id, response.as_bytes());

                    if let Some(player) = self.world.players.get(&player_id) {
                        let f = player.get_food_count(self.world.freq);
                        let l = player.inventory.get(&Resource::Linemate).unwrap_or(&0);
                        let d = player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
                        let s = player.inventory.get(&Resource::Sibur).unwrap_or(&0);
                        let m = player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
                        let p = player.inventory.get(&Resource::Phiras).unwrap_or(&0);
                        let t = player.inventory.get(&Resource::Thystame).unwrap_or(&0);
                        let orientation = match player.direction {
                            crate::game::player::Direction::North => 1,
                            crate::game::player::Direction::East => 2,
                            crate::game::player::Direction::South => 3,
                            crate::game::player::Direction::West => 4,
                        };
                        self.world.gui_events.push_back(format!("pnw #{} {} {} {} {} {}\n", player_id, player.x, player.y, orientation, player.level, player.team));
                        self.world.gui_events.push_back(format!("pin #{} {} {} {} {} {} {} {} {} {}\n", player_id, player.x, player.y, f, l, d, s, m, p, t));
                    }
                    
                    self.log(format!("Player {} joined team '{}'", player_id, msg));
                    self.emit_event(ServerEvent::PlayerJoinedTeam(msg.clone()));
                } else {
                    network.send_message(client_id, b"ko\n");
                }
            }
            Some(1) => {
                let id = if let Some(ClientState::InGame(pid)) = self.client_states.get(&client_id) { *pid } else { return; };
                if let Some(cmd) = Command::from_str(&msg) {
                    if let Some(response) = crate::game::commands::init(&cmd, id, &mut self.world) {
                        network.send_message(client_id, response.as_bytes());
                    }

                    if let Some(player) = self.world.players.get_mut(&id) {
                        if player.commands.len() < 10 {
                            let duration = Duration::from_secs_f64(cmd.duration() as f64 / self.config.freq as f64);
                            let start = player.last_command_end.max(Instant::now());
                            let end = start + duration;
                            player.last_command_end = end;
                            player.commands.push_back(PendingCommand { command: cmd, end_time: end });
                        }
                    }
                } else {
                    network.send_message(client_id, b"ko\n");
                }
            }
            Some(2) => {
                if let Some(cmd) = GuiCommand::from_str(&msg) {
                    self.handle_gui_command(client_id, cmd, network);
                } else {
                    network.send_message(client_id, b"suc\n");
                }
            }
            _ => {}
        }
    }

    fn on_tick(&mut self, network: &mut NetworkServer) -> bool {
        let now = Instant::now();

        if now >= self.world.next_spawn_time {
            self.world.spawn_resources();
            let spawn_interval = Duration::from_secs_f64(20.0 / self.config.freq as f64);
            self.world.next_spawn_time = now + spawn_interval;
        }

        let mut dead_players = Vec::new();
        let mut commands_to_execute = Vec::new();

        for (client_id, state) in &self.client_states {
            if let ClientState::InGame(id) = state {
                if let Some(player) = self.world.players.get_mut(id) {
                    if now >= player.death_time {
                        network.send_message(*client_id, b"dead\n");
                        dead_players.push((*client_id, *id));
                        continue;
                    }

                    let current_food = player.get_food_count(self.world.freq);
                    if current_food != player.last_food_count {
                        player.last_food_count = current_food;
                        let l = *player.inventory.get(&Resource::Linemate).unwrap_or(&0);
                        let d = *player.inventory.get(&Resource::Deraumere).unwrap_or(&0);
                        let s = *player.inventory.get(&Resource::Sibur).unwrap_or(&0);
                        let m = *player.inventory.get(&Resource::Mendiane).unwrap_or(&0);
                        let p = *player.inventory.get(&Resource::Phiras).unwrap_or(&0);
                        let t = *player.inventory.get(&Resource::Thystame).unwrap_or(&0);
                        self.world.gui_events.push_back(format!(
                            "pin #{} {} {} {} {} {} {} {} {} {}\n",
                            id, player.x, player.y, current_food, l, d, s, m, p, t
                        ));
                    }

                    while let Some(cmd) = player.commands.front() {
                        if now >= cmd.end_time {
                            if let Some(pending) = player.commands.pop_front() {
                                commands_to_execute.push((*client_id, pending.command));
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        for (client_id, cmd) in commands_to_execute {
            self.execute_command(client_id, cmd, network);
        }

        for (client_id, id) in dead_players {
            self.broadcast_gui(&format!("pdi #{}\n", id), network);
            let team = self.world.players.get(&id).map(|p| p.team.clone()).unwrap_or_default();
            self.log(format!("Player {} (Team: {}) starved to death.", id, team));
            self.emit_event(ServerEvent::PlayerDied(team));

            self.world.remove_player(id);
            network.disconnect_client(client_id);
            self.client_states.remove(&client_id);
            self.emit_event(ServerEvent::ClientDisconnected);
        }

        let mut dead_eggs = Vec::new();
        let mut i = 0;
        while i < self.world.eggs.len() {
            if now >= self.world.eggs[i].death_time {
                let dead_egg = self.world.eggs.remove(i);
                dead_eggs.push(dead_egg.id);
            } else {
                i += 1;
            }
        }
        for egg_id in dead_eggs {
            self.broadcast_gui(&format!("edi #{}\n", egg_id), network);
        }

        let mut world_events = Vec::new();
        while let Some(event) = self.world.gui_events.pop_front() {
            world_events.push(event);
        }
        for event in world_events {
            self.broadcast_gui(&event, network);
        }

        self.flush_notifications(network);

        if let Some(winning_team) = self.world.check_victory() {
            self.broadcast_gui(&format!("seg {}\n", winning_team), network);
            self.log(format!("Game Over! Team {} wins!", winning_team));
            self.emit_event(ServerEvent::GameOver(winning_team));
            return true;
        }

        let mut positions = Vec::new();
        for player in self.world.players.values() {
            positions.push((player.x, player.y));
        }
        self.emit_event(ServerEvent::MapSnapshot(positions));

        false
    }

    fn get_next_timeout(&self) -> Duration {
        let now = Instant::now();
        let mut min_time = now + Duration::from_millis(100);

        for player in self.world.players.values() {
            if let Some(cmd) = player.commands.front() {
                if cmd.end_time < min_time {
                    min_time = cmd.end_time;
                }
            }
            if player.death_time < min_time {
                min_time = player.death_time;
            }
        }

        for egg in &self.world.eggs {
            if egg.death_time < min_time {
                min_time = egg.death_time;
            }
        }

        if self.world.next_spawn_time < min_time {
            min_time = self.world.next_spawn_time;
        }

        min_time.saturating_duration_since(now)
    }
}
