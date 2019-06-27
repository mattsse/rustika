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

use reqwest::{IntoUrl, Url};
use std::net;

/// Indicates whether a tika server instance should spawned or is already running
#[derive(Debug, Clone)]
pub enum TikaMode {
    /// also start the tika server and bind it to the address
    /// by default the server runs at `127.0.0.1:9998`
    ClientServer(net::SocketAddr),
    /// don't start a tika server instead access an already running server reachable via `Url`
    ClientOnly(Url),
}

impl TikaMode {
    /// A tika server should be spawned at a local address
    #[inline]
    pub fn client_server<T: AsRef<str>>(addr: T) -> Result<Self> {
        Ok(TikaMode::ClientServer(addr.as_ref().parse()?))
    }

    /// the url of the tika server, either local and self hosted or remote
    pub fn server_endpoint(&self) -> Url {
        match self {
            TikaMode::ClientServer(addr) => Url::parse(&format!("http://{}", addr)).unwrap(),
            TikaMode::ClientOnly(url) => url.clone(),
        }
    }

    /// Creates a `TikaMode::ClientOnly` with the desired `server_url` as tika server endpoint
    pub fn client_only<U: IntoUrl>(server_url: U) -> Result<Self> {
        Ok(TikaMode::ClientOnly(server_url.into_url()?))
    }
}

impl Default for TikaMode {
    fn default() -> Self {
        if let Ok(url) = ::std::env::var("TIKA_SERVER_ENDPOINT") {
            TikaMode::ClientOnly(
                Url::parse(&url)
                    .unwrap_or_else(|_| panic!("Failed to convert {} to a valid url", url)),
            )
        } else {
            TikaMode::ClientServer(net::SocketAddr::new(
                net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)),
                9998,
            ))
        }
    }
}
