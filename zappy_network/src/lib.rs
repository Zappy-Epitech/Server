//! Zappy Network Crate
//!
//! Provides the core networking capabilities for the Zappy server using Mio for non-blocking I/O.

/// Module containing the circular buffer implementation.
pub mod buffer;
/// Module containing the core network server logic.
pub mod network;

pub use network::{NetworkServer, ServerEventHandler};
pub use buffer::CircularBuffer;
