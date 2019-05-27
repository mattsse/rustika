#[macro_use]
extern crate serde;
#[macro_use]
extern crate log;

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
    pub fn client_server<T: AsRef<str>>(addr: T) -> Result<Self> {
        Ok(TikaMode::ClientServer(addr.as_ref().parse()?))
    }

    pub fn server_endpoint(&self) -> Url {
        match self {
            TikaMode::ClientServer(addr) => Url::parse(&format!("http://{}", addr)).unwrap(),
            TikaMode::ClientOnly(url) => url.clone(),
        }
    }
}

impl Default for TikaMode {
    fn default() -> Self {
        if let Ok(url) = ::std::env::var("TIKA_SERVER_ENDPOINT") {
            TikaMode::ClientOnly(
                Url::parse(&url).expect(&format!("Failed to convert {} to a valid url", url)),
            )
        } else {
            TikaMode::ClientServer(net::SocketAddr::new(
                net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)),
                9998,
            ))
        }
    }
}
