#[macro_use]
extern crate serde;

pub mod client;
mod error;
pub mod server;
pub mod web;

pub use crate::client::{TikaBuilder, TikaClient};
pub use crate::error::Result;

#[derive(Debug, Clone)]
pub enum TikaMode {
    ClientServer,
    ClientOnly,
    ServerOnly,
}

impl Default for TikaMode {
    fn default() -> Self {
        TikaMode::ClientServer
    }
}
