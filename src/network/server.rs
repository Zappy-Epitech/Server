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
}

impl Server {
    /// Creates a new Server instance based on the provided configuration.
    pub fn new(config: ServerConfig) -> io::Result<Self> {
        let poll = Poll::new()?;
        let world = World::new(
            config.width,
            config.height,
            config.teams.clone(),
            config.clients_nb,
        );

        Ok(Self {
            config,
            poll,
            clients: HashMap::new(),
            next_token: 1,
            world,
        })
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
                        while let Ok((mut stream, _)) = listener.accept() {
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
            self.update_game();
        }
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

        min_time.saturating_duration_since(now)
    }

    fn update_game(&mut self) {
        let now = Instant::now();
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
            self.world.remove_player(id);
            self.clients.remove(&token);
        }
    }

    fn execute_command(&mut self, token: Token, cmd: Command) {
        if let Some(client) = self.clients.get_mut(&token) {
            match cmd {
                _ => {
                    client.buffer_out.extend_from_slice(b"ok\n");
                }
            }
        }
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
            self.clients.remove(&token);
            if let Some(id) = player_to_remove {
                self.world.remove_player(id);
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
                        } else if let Some(player_id) = self.world.add_player(&line_str, self.config.freq) {
                            client.state = ClientState::InGame(player_id);
                            client.team_name = Some(line_str.clone());
                            let slots = self.world.team_slots.get(&line_str).unwrap_or(&0);
                            let msg = format!("{}\n{} {}\n", slots, self.config.width, self.config.height);
                            client.buffer_out.extend_from_slice(msg.as_bytes());
                        } else {
                            client.buffer_out.extend_from_slice(b"ko\n");
                        }
                    }
                    ClientState::InGame(id) => {
                        if let Some(cmd) = Command::from_str(&line_str) {
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
                        }
                    }
                    _ => {}
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
