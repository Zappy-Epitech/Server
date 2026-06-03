use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

use crate::config::ServerConfig;
use crate::network::client::{Client, ClientState};

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
}

impl Server {
    /// Creates a new Server instance based on the provided configuration.
    pub fn new(config: ServerConfig) -> io::Result<Self> {
        let poll = Poll::new()?;

        Ok(Self {
            config,
            poll,
            clients: HashMap::new(),
            next_token: 1,
        })
    }

    /// Starts the main event loop of the server.
    /// 
    /// This method will block and handle new connections, incoming data,
    /// and outgoing data until an error occurs.
    pub fn run(&mut self) -> io::Result<()> {
        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port).parse().unwrap();
        let mut listener = TcpListener::bind(addr)?;

        self.poll.registry().register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;

        let mut events = Events::with_capacity(128);

        loop {
            self.poll.poll(&mut events, Some(Duration::from_millis(100)))?;

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
        }
    }

    /// Reads data from a client's socket into its input buffer.
    fn handle_read(&mut self, token: Token) {
        let mut closed = false;
        if let Some(client) = self.clients.get_mut(&token) {
            let mut buf = [0; 1024];
            loop {
                match client.stream.read(&mut buf) {
                    Ok(0) => {
                        closed = true;
                        break;
                    }
                    Ok(n) => client.buffer_in.extend_from_slice(&buf[..n]),
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(_) => {
                        closed = true;
                        break;
                    }
                }
            }
        }

        if closed {
            self.clients.remove(&token);
            return;
        }

        self.process_buffer(token);
    }

    /// Processes the input buffer of a client, extracting complete lines (commands).
    fn process_buffer(&mut self, token: Token) {
        if let Some(client) = self.clients.get_mut(&token) {
            while let Some(pos) = client.buffer_in.iter().position(|&b| b == b'\n') {
                let line = client.buffer_in.drain(..pos + 1).collect::<Vec<u8>>();
                let line_str = String::from_utf8_lossy(&line).trim().to_string();

                match client.state {
                    ClientState::Authenticating => {
                        if line_str == "GRAPHIC" {
                            client.state = ClientState::Graphic;
                        } else if self.config.teams.contains(&line_str) {
                            client.state = ClientState::InGame;
                            client.team_name = Some(line_str.clone());
                            let msg = format!("{}\n{} {}\n", self.config.clients_nb, self.config.width, self.config.height);
                            client.buffer_out.extend_from_slice(msg.as_bytes());
                        } else {
                            client.buffer_out.extend_from_slice(b"ko\n");
                        }
                    }
                    _ => {
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
