//! Core game simulation.
//!
//! Groups and re-exports the engine submodules: [`world`] (the toroidal map,
//! players, eggs and resource logic), [`player`] (per-drone state and hunger),
//! and [`commands`] (the time-delayed command initiation and execution
//! pipeline).

pub mod world;
pub mod player;
pub mod commands;
