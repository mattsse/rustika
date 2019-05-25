#[macro_use]
extern crate serde;

pub mod client;
mod error;
pub mod server;
pub mod web;

pub use crate::client::{TikaBuilder, TikaClient};
pub use crate::error::Result;

use reqwest::Url;
use std::net;

#[derive(Debug, Clone)]
pub enum TikaMode {
    /// also start the tika server and bind it to the address
    /// by default the server runs at `127.0.0.1:9998`
    ClientServer(net::SocketAddr),
    /// don't start a tika server instead access an already running server reachable via `Url`
    ClientOnly(Url),
}

impl TikaMode {
    #[inline]
    pub fn client_server(addr: &str) -> Result<Self> {
        Ok(TikaMode::ClientServer(addr.parse()?))
    }
}

impl Default for TikaMode {
    fn default() -> Self {
        TikaMode::ClientServer(net::SocketAddr::new(
            net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)),
            9998,
        ))
    }
}
