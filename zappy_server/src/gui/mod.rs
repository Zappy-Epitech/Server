//! GUI response generation.
//!
//! Houses the [`commands`] submodule that turns parsed
//! [`GuiCommand`](crate::protocol::gui::GuiCommand)s into the protocol lines
//! sent back to graphic observers.

pub mod commands;
