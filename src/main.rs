mod config;
mod network;
mod game;
mod protocol;
mod gui;
mod tui;

use clap::Parser;
use config::ServerConfig;
use network::server::Server;
use std::io;

fn main() -> io::Result<()> {
    let config = ServerConfig::parse();

    if config.bonus {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut server = Server::new(config.clone(), Some(tx))?;
        
        std::thread::spawn(move || {
            let _ = server.run();
        });

        return tui::app::run(rx, &config);
    }

    let mut server = Server::new(config, None)?;
    server.run()
}

