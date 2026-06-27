use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::net::SocketAddr;
use std::time::Duration;

use crate::buffer::CircularBuffer;

const SERVER_TOKEN: Token = Token(0);

/// Represents a connected client in the network layer.
pub struct NetworkClient {
    /// The underlying TCP stream of the client.
    pub stream: TcpStream,
    /// The incoming data buffer.
    pub buffer_in: CircularBuffer,
    /// The outgoing data buffer.
    pub buffer_out: CircularBuffer,
}

impl NetworkClient {
    /// Creates a new `NetworkClient` from a raw TCP stream.
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer_in: CircularBuffer::new(),
            buffer_out: CircularBuffer::new(),
        }
    }
}

/// Trait implemented by the application layer to react to network events.
pub trait ServerEventHandler {
    /// Called when a new client connects.
    fn on_client_connected(&mut self, client_id: usize, network: &mut NetworkServer);
    /// Called when a client disconnects.
    fn on_client_disconnected(&mut self, client_id: usize, network: &mut NetworkServer);
    /// Called when a complete message (terminated by \n) is received.
    fn on_message_received(&mut self, client_id: usize, msg: String, network: &mut NetworkServer);
    /// Called regularly on each tick of the event loop. Returns true to stop the server.
    fn on_tick(&mut self, network: &mut NetworkServer) -> bool;
    /// Returns the duration until the next game event timeout.
    fn get_next_timeout(&self) -> Duration;
}

/// The core non-blocking network server using Mio.
pub struct NetworkServer {
    poll: Poll,
    clients: HashMap<usize, NetworkClient>,
    next_client_id: usize,
    listener_bound: bool,
}

impl NetworkServer {
    /// Creates a new `NetworkServer` instance with an initialized polling mechanism.
    pub fn new() -> io::Result<Self> {
        let poll = Poll::new()?;
        Ok(Self {
            poll,
            clients: HashMap::new(),
            next_client_id: 1,
            listener_bound: false,
        })
    }

    /// Queues a message to be sent to a specific client.
    pub fn send_message(&mut self, client_id: usize, msg: &[u8]) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            let _ = client.buffer_out.push(msg);
            self.register_write(client_id);
        }
    }

    /// Forcibly disconnects a client.
    pub fn disconnect_client(&mut self, client_id: usize) {
        if let Some(mut client) = self.clients.remove(&client_id) {
            let _ = self.poll.registry().deregister(&mut client.stream);
        }
    }

    fn register_write(&mut self, client_id: usize) {
        if let Some(client) = self.clients.get_mut(&client_id) {
            let token = Token(client_id);
            let _ = self.poll.registry().reregister(
                &mut client.stream,
                token,
                Interest::READABLE | Interest::WRITABLE,
            );
        }
    }


    /// Starts the main non-blocking event loop.
    pub fn run<H: ServerEventHandler>(&mut self, port: u16, handler: &mut H) -> io::Result<()> {
        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let mut listener = match TcpListener::bind(addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("NetworkServer: Failed to bind to port {}: {}", port, e);
                return Err(e);
            }
        };

        self.poll.registry().register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;
        self.listener_bound = true;

        let mut events = Events::with_capacity(128);

        loop {
            let timeout = handler.get_next_timeout();
            if let Err(e) = self.poll.poll(&mut events, Some(timeout)) {
                if e.kind() == ErrorKind::Interrupted { continue; }
                eprintln!("NetworkServer: Poll error: {}", e);
                break;
            }

            let mut disconnected_clients = Vec::new();
            let mut newly_connected = Vec::new();

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        while let Ok((mut stream, _addr)) = listener.accept() {
                            let client_id = self.next_client_id;
                            self.next_client_id += 1;

                            if let Err(e) = self.poll.registry().register(
                                &mut stream,
                                Token(client_id),
                                Interest::READABLE,
                            ) {
                                eprintln!("NetworkServer: Failed to register new client stream: {}", e);
                                continue;
                            }

                            let client = NetworkClient::new(stream);
                            self.clients.insert(client_id, client);
                            newly_connected.push(client_id);
                        }
                    }
                    Token(id) => {
                        if event.is_readable() {
                            if let Some(client) = self.clients.get_mut(&id) {
                                match client.buffer_in.read_from(&mut client.stream) {
                                    Ok(0) => {
                                        disconnected_clients.push(id);
                                        continue;
                                    }
                                    Ok(_) => {}
                                    Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                                    Err(e) if e.kind() == ErrorKind::OutOfMemory => {
                                        eprintln!("NetworkServer: Client {} disconnected due to buffer overflow", id);
                                        disconnected_clients.push(id);
                                        continue;
                                    }
                                    Err(_) => {
                                        disconnected_clients.push(id);
                                        continue;
                                    }
                                }
                            }
                        }
                        if event.is_writable() {
                            if let Some(client) = self.clients.get_mut(&id) {
                                match client.buffer_out.write_to(&mut client.stream) {
                                    Ok(_) => {
                                        if client.buffer_out.is_empty() {
                                            let token = Token(id);
                                            let _ = self.poll.registry().reregister(
                                                &mut client.stream,
                                                token,
                                                Interest::READABLE,
                                            );
                                        }
                                    }
                                    Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                                    Err(_) => {
                                        disconnected_clients.push(id);
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            for id in newly_connected {
                handler.on_client_connected(id, self);
            }

            for id in &disconnected_clients {
                self.disconnect_client(*id);
                handler.on_client_disconnected(*id, self);
            }

            let mut client_messages = Vec::new();
            for (id, client) in self.clients.iter_mut() {
                while let Some(line) = client.buffer_in.extract_line() {
                    client_messages.push((*id, line));
                }
            }

            for (id, msg) in client_messages {
                if self.clients.contains_key(&id) {
                    handler.on_message_received(id, msg, self);
                }
            }

            if handler.on_tick(self) {
                break;
            }
        }
        Ok(())
    }
}
