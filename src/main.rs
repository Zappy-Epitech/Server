mod config;
mod network;
mod game;
mod protocol;
mod gui;

use clap::Parser;
use config::ServerConfig;
use network::server::Server;
use std::io;

fn main() -> io::Result<()> {
    let config = ServerConfig::parse();
    let mut server = Server::new(config)?;
    
    server.run()
}
