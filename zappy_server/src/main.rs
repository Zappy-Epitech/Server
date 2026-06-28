//! # Zappy Server
//!
//! A single-process, non-blocking game server for *Zappy*: AI drones (the
//! "Trantorians") roam a toroidal world, gather resources and perform elevation
//! rituals to climb from level 1 to level 8. A team wins once six of its players
//! reach the maximum level.
//!
//! ## Design at a glance
//!
//! The whole simulation runs in one thread on top of a [Mio](https://docs.rs/mio)
//! `Poll` event loop (provided by the [`zappy_network`] crate). There is no
//! per-client thread and no blocking I/O: sockets are line-buffered and serviced
//! as they become readable/writable, and time progresses through a `freq`-scaled
//! command queue rather than real wall-clock ticks.
//!
//! Two client protocols share the same listener:
//! * **AI clients** authenticate with a team name, then issue game commands that
//!   are queued and executed after a time delay proportional to their cost.
//! * **GUI observers** authenticate with the literal `GRAPHIC` and receive a live
//!   broadcast of every world event for visualization.
//!
//! With `--bonus`, the server thread is spawned in the background and the main
//! thread renders a [Ratatui] TUI dashboard fed by an `mpsc` channel of
//! [`ServerEvent`](tui::ServerEvent)s.
//!
//! ## Architecture
//!
//! ```text
//!                AI clients          GUI observers
//!                     |                    |
//!                     v                    v
//!            +-----------------------------------------+
//!            |   zappy_network::NetworkServer (Mio)    |   <- non-blocking TCP,
//!            |   poll(): accept / READABLE / WRITABLE  |      line-buffered I/O
//!            +-----------------------------------------+
//!                     |                         ^
//!         on_message_received               on_tick (every loop)
//!                     |                         |
//!                     v                         |
//!            +-----------------------------------------+
//!            |   network::server::Server               |
//!            |   (ServerEventHandler impl)             |
//!            |   ClientState: Authenticating /         |
//!            |                InGame(id) / Graphic     |
//!            +--------------------+--------------------+
//!               AI: parse Command |  GUI: parse GuiCommand
//!               queue PendingCmd  |  gui::commands::execute
//!                     |           |          |
//!                     v           v          |
//!            +-----------------------------+ |
//!            |   game::World               | |
//!            |   tiles (toroidal) players  | |
//!            |   eggs, resources, freq     | |
//!            |   gui_events VecDeque  -----+-+--> broadcast to all GUI clients
//!            +-----------------------------+
//!                     |
//!                  on_tick: respawn / hunger / run due commands /
//!                           egg expiry / victory check
//!                     |
//!                     v (ServerEvent via mpsc, --bonus only)
//!            +-----------------------------+
//!            |   tui::app  (Ratatui TUI)   |
//!            +-----------------------------+
//! ```
//!
//! [Ratatui]: https://docs.rs/ratatui

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
    config.validate();

    if config.bonus {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut server = Server::new(config.clone(), Some(tx));
        
        std::thread::spawn(move || {
            let _ = server.run();
        });

        return tui::app::run(rx, &config);
    }

    let mut server = Server::new(config, None);
    server.run()
}

