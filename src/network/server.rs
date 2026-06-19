use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::config::ServerConfig;
use crate::network::client::{Client, ClientState};
use crate::game::world::World;
use crate::protocol::{Command, PendingCommand};
use crate::protocol::gui::GuiCommand;
use crate::tui::ServerEvent;
use std::sync::mpsc::Sender;

/// Token used to identify the server listener in the event loop.
const SERVER_TOKEN: Token = Token(0);

/// The core server structure managing network connections and the event loop.
pub struct Server {
    /// Configuration parameters for the server.
    config: ServerConfig,
    /// The Mio poll instance for monitoring network events.
    poll: Poll,
    /// A map of connected clients, indexed by their unique Token.
    clients: HashMap<Token, Client>,
    /// Counter used to generate the next unique Token for a new client.
    next_token: usize,
    /// The game world of Trantor.
    world: World,
    /// Optional sender to emit events to the TUI.
    tui_tx: Option<Sender<ServerEvent>>,
}

impl Server {
    /// Creates a new Server instance based on the provided configuration.
    pub fn new(config: ServerConfig, tui_tx: Option<Sender<ServerEvent>>) -> io::Result<Self> {
        let poll = Poll::new()?;
        let world = World::new(
            config.width,
            config.height,
            config.teams.clone(),
            config.clients_nb,
            config.freq,
        );

        Ok(Self {
            config,
            poll,
            clients: HashMap::new(),
            next_token: 1,
            world,
            tui_tx,
        })
    }

    /// Helper to either print to stdout or send to TUI.
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

    /// Starts the main event loop of the server.
    pub fn run(&mut self) -> io::Result<()> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port).parse().unwrap();
        let mut listener = TcpListener::bind(addr)?;

        self.poll.registry().register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

        let mut events = Events::with_capacity(128);

        loop {
            let timeout = self.get_next_timeout();
            self.poll.poll(&mut events, Some(timeout))?;

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        while let Ok((mut stream, addr)) = listener.accept() {
                            let token = Token(self.next_token);
                            self.next_token += 1;

                            self.poll.registry().register(
                                &mut stream,
                                token,
                                Interest::READABLE | Interest::WRITABLE,
                            )?;

                            let mut client = Client::new(stream);
                            client.buffer_out.extend_from_slice(b"WELCOME\n");
                            self.clients.insert(token, client);
                            self.handle_write(token);

                            self.log(format!("New connection from {}", addr));
                            self.emit_event(ServerEvent::ClientConnected);
                        }
                    }
                    token => {
                        if event.is_readable() {
                            self.handle_read(token);
                        }
                        if event.is_writable() {
                            self.handle_write(token);
                        }
                    }
                }
            }
            if self.update_game() {
                self.flush_notifications();
                break;
            }
            self.flush_notifications();
        }
        Ok(())
    }

    /// Sends all pending notifications to the clients.
    fn flush_notifications(&mut self) {
        let mut tokens_to_flush = Vec::new();
        for (token, client) in self.clients.iter_mut() {
            if let ClientState::InGame(id) = client.state {
                if let Some(player) = self.world.players.get_mut(&id) {
                    while let Some(notif) = player.notifications.pop_front() {
                        client.buffer_out.extend_from_slice(notif.as_bytes());
                        tokens_to_flush.push(*token);
                    }
                }
            }
        }
        for token in tokens_to_flush {
            self.handle_write(token);
        }
    }

    /// Sends a message to all connected graphical clients.
    pub fn broadcast_gui(&mut self, msg: &str) {
        let mut tokens = Vec::new();
        for (token, client) in self.clients.iter_mut() {
            if let ClientState::Graphic = client.state {
                client.buffer_out.extend_from_slice(msg.as_bytes());
                tokens.push(*token);
            }
        }
        for token in tokens {
            self.handle_write(token);
        }
    }

    /// Calculates the duration until the next game event.
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

        if self.world.next_spawn_time < min_time {
            min_time = self.world.next_spawn_time;
        }

        min_time.saturating_duration_since(now)
    }

    /// Checks for expired commands, player deaths, and resource respawn.
    fn update_game(&mut self) -> bool {
        let now = Instant::now();

        if now >= self.world.next_spawn_time {
            self.world.spawn_resources();
            let spawn_interval = Duration::from_secs_f64(20.0 / self.config.freq as f64);
            self.world.next_spawn_time = now + spawn_interval;
        }

        let mut dead_players = Vec::new();
        let mut commands_to_execute = Vec::new();

        for (token, client) in self.clients.iter_mut() {
            if let ClientState::InGame(id) = client.state {
                if let Some(player) = self.world.players.get_mut(&id) {
                    if now >= player.death_time {
                        client.buffer_out.extend_from_slice(b"dead\n");
                        dead_players.push((*token, id));
                        continue;
                    }

                    while let Some(cmd) = player.commands.front() {
                        if now >= cmd.end_time {
                            let pending = player.commands.pop_front().unwrap();
                            commands_to_execute.push((*token, pending.command));
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        for (token, cmd) in commands_to_execute {
            self.execute_command(token, cmd);
        }

        for (token, id) in dead_players {
            self.broadcast_gui(&format!("pdi {}\n", id));
            let team = self.world.players.get(&id).map(|p| p.team.clone()).unwrap_or_default();
            self.log(format!("Player {} (Team: {}) starved to death.", id, team));
            self.emit_event(ServerEvent::PlayerDied(team));

            if self.clients.contains_key(&token) {
                self.handle_write(token);
            }
            self.world.remove_player(id);
            self.clients.remove(&token);
            self.emit_event(ServerEvent::ClientDisconnected);
        }

        let mut world_events = Vec::new();
        while let Some(event) = self.world.gui_events.pop_front() {
            world_events.push(event);
        }
        for event in world_events {
            self.broadcast_gui(&event);
        }

        if let Some(winning_team) = self.world.check_victory() {
            self.broadcast_gui(&format!("seg {}\n", winning_team));
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

    /// Logic to execute a command after its duration has elapsed.
    fn execute_command(&mut self, token: Token, cmd: Command) {
        let player_id = if let Some(client) = self.clients.get(&token) {
            if let ClientState::InGame(id) = client.state {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(id) = player_id {
            let response = crate::game::commands::execute(cmd, id, &mut self.world);
            if let Some(client) = self.clients.get_mut(&token) {
                client.buffer_out.extend_from_slice(response.as_bytes());
            }
            self.handle_write(token);
        }
    }

    fn handle_gui_command(&mut self, token: Token, cmd: GuiCommand) {
        let responses = crate::gui::commands::execute(cmd.clone(), &mut self.world, &mut self.config);
        
        if let GuiCommand::TimeUpdate(_) = cmd {
            self.log(format!("GUI modified frequency to {}", self.config.freq));
            self.emit_event(ServerEvent::FreqChanged(self.config.freq));
        }

        if let Some(client) = self.clients.get_mut(&token) {
            for response in responses {
                client.buffer_out.extend_from_slice(response.as_bytes());
            }
        }
        self.handle_write(token);
    }

    /// Reads data from a client's socket into its input buffer.
    fn handle_read(&mut self, token: Token) {
        let mut closed = false;
        let mut player_to_remove = None;

        if let Some(client) = self.clients.get_mut(&token) {
            let mut buf = [0; 1024];
            loop {
                match client.stream.read(&mut buf) {
                    Ok(0) => {
                        closed = true;
                        if let ClientState::InGame(id) = client.state {
                            player_to_remove = Some(id);
                        }
                        break;
                    }
                    Ok(n) => client.buffer_in.extend_from_slice(&buf[..n]),
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(_) => {
                        closed = true;
                        if let ClientState::InGame(id) = client.state {
                            player_to_remove = Some(id);
                        }
                        break;
                    }
                }
            }
        }

        if closed {
            self.log(format!("Client disconnected (Token: {:?})", token));
            self.emit_event(ServerEvent::ClientDisconnected);
            self.clients.remove(&token);
            if let Some(id) = player_to_remove {
                self.broadcast_gui(&format!("pdi {}\n", id));
                let team = self.world.players.get(&id).map(|p| p.team.clone()).unwrap_or_default();
                self.world.remove_player(id);
                self.emit_event(ServerEvent::PlayerDied(team));
            }
            return;
        }

        self.process_buffer(token);
    }

    /// Processes the input buffer of a client, extracting complete lines (commands).
    fn process_buffer(&mut self, token: Token) {
        let mut lines = Vec::new();
        if let Some(client) = self.clients.get_mut(&token) {
            while let Some(pos) = client.buffer_in.iter().position(|&b| b == b'\n') {
                let line = client.buffer_in.drain(..pos + 1).collect::<Vec<u8>>();
                lines.push(String::from_utf8_lossy(&line).trim().to_string());
            }
        }

        for line_str in lines {
            if let Some(client) = self.clients.get_mut(&token) {
                match client.state {
                    ClientState::Authenticating => {
                        if line_str == "GRAPHIC" {
                            client.state = ClientState::Graphic;
                            client.buffer_out.extend_from_slice(b"smg Welcome to Zappy Server!\n");
                            client.buffer_out.extend_from_slice(format!("msz {} {}\n", self.config.width, self.config.height).as_bytes());
                            client.buffer_out.extend_from_slice(format!("sgt {}\n", self.config.freq).as_bytes());

                            for y in 0..self.config.height {
                                for x in 0..self.config.width {
                                    client.buffer_out.extend_from_slice(crate::gui::commands::map::format_bct(&self.world, x, y).as_bytes());
                                }
                            }

                            for team in &self.config.teams {
                                client.buffer_out.extend_from_slice(format!("tna {}\n", team).as_bytes());
                            }

                            for p in self.world.players.values() {
                                let o = match p.direction {
                                    crate::game::player::Direction::North => 1, crate::game::player::Direction::East => 2,
                                    crate::game::player::Direction::South => 3, crate::game::player::Direction::West => 4,
                                };
                                client.buffer_out.extend_from_slice(format!("pnw {} {} {} {} {} {}\n", p.id, p.x, p.y, o, p.level, p.team).as_bytes());
                            }
                            for e in &self.world.eggs {
                                client.buffer_out.extend_from_slice(format!("enw {} 0 {} {}\n", e.id, e.x, e.y).as_bytes());
                            }
                            self.log(format!("Graphic client connected (Token: {:?})", token));
                        } else if let Some(player_id) = self.world.add_player(&line_str, self.config.freq) {
                            client.state = ClientState::InGame(player_id);
                            client.team_name = Some(line_str.clone());
                            let initial_slots = *self.world.team_slots.get(&line_str).unwrap_or(&0);
                            let egg_slots = self.world.eggs.iter().filter(|e| e.team == line_str).count();
                            let msg = format!("{}\n{} {}\n", initial_slots + egg_slots, self.config.width, self.config.height);
                            client.buffer_out.extend_from_slice(msg.as_bytes());
                            
                            let player = self.world.players.get(&player_id).unwrap();
                            let orientation = match player.direction {
                                crate::game::player::Direction::North => 1,
                                crate::game::player::Direction::East => 2,
                                crate::game::player::Direction::South => 3,
                                crate::game::player::Direction::West => 4,
                            };
                            self.broadcast_gui(&format!("pnw {} {} {} {} {} {}\n", player_id, player.x, player.y, orientation, player.level, player.team));
                            
                            self.log(format!("Player {} joined team '{}'", player_id, line_str));
                            self.emit_event(ServerEvent::PlayerJoinedTeam(line_str.clone()));
                        } else {
                            client.buffer_out.extend_from_slice(b"ko\n");
                        }
                        self.handle_write(token);
                    }
                    ClientState::InGame(id) => {
                        if let Some(cmd) = Command::from_str(&line_str) {
                            if let Some(msg) = crate::game::commands::init(&cmd, id, &mut self.world) {
                                client.buffer_out.extend_from_slice(msg.as_bytes());
                                self.handle_write(token);
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
                            client.buffer_out.extend_from_slice(b"ko\n");
                            self.handle_write(token);
                        }
                    }
                    ClientState::Graphic => {
                        if let Some(cmd) = GuiCommand::from_str(&line_str) {
                            self.handle_gui_command(token, cmd);
                        } else {
                            client.buffer_out.extend_from_slice(b"suc\n");
                            self.handle_write(token);
                        }
                    }
                }
            }
        }
    }

    /// Writes data from a client's output buffer to its socket.
    fn handle_write(&mut self, token: Token) {
        if let Some(client) = self.clients.get_mut(&token) {
            if !client.buffer_out.is_empty() {
                match client.stream.write(&client.buffer_out) {
                    Ok(n) => {
                        client.buffer_out.drain(..n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => {
                        self.clients.remove(&token);
                    }
                }
            }
        }
    }
}
