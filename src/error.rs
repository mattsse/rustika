use failure::{Backtrace, Context, Fail};
use std::{fmt, result};

/// A type alias for handling errors throughout crossref.
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur while interacting while handling rustika.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Error {
    ctx: Context<ErrorKind>,
}

impl Error {
    pub(crate) fn config<T: AsRef<str>>(msg: T) -> Error {
        Error::from(ErrorKind::Config {
            msg: msg.as_ref().to_string(),
        })
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.ctx.fmt(f)
    }
}

/// all different error types this crate uses
#[derive(Debug, Fail)]
#[allow(missing_docs)]
pub enum ErrorKind {
    /// if an invalid type was requested
    #[fail(display = "invalid type name: {}", name)]
    InvalidTypeName { name: String },

    /// a config error
    #[fail(display = "{}", msg)]
    Config {
        /// the notification
        msg: String,
    },

    #[fail(display = "Failed during std::io operation: {}", io)]
    IO { io: std::io::Error },

    /// an error that occurred while operating with [reqwest]
    #[fail(display = "{}", reqwest)]
    ReqWest { reqwest: reqwest::Error },
    /// if a error in serde occurred
    #[fail(display = "invalid serde: {}", error)]
    Serde { error: serde_json::Error },

    #[fail(display = "Failed to parse url: {}", url)]
    Url { url: reqwest::UrlError },
    // TODO unify to single parse error
    #[fail(display = "Failed to parse address: {}", addr)]
    Addr { addr: std::net::AddrParseError },
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error::from(Context::new(kind))
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(ctx: Context<ErrorKind>) -> Error {
        Error { ctx }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        ErrorKind::Serde { error }.into()
    }
}

impl From<reqwest::Error> for Error {
    fn from(reqwest: reqwest::Error) -> Error {
        ErrorKind::ReqWest { reqwest }.into()
    }
}

impl From<reqwest::UrlError> for Error {
    fn from(url: reqwest::UrlError) -> Error {
        ErrorKind::Url { url }.into()
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(addr: std::net::AddrParseError) -> Error {
        ErrorKind::Addr { addr }.into()
    }
}

impl From<std::io::Error> for Error {
    fn from(io: std::io::Error) -> Error {
        ErrorKind::IO { io }.into()
    }
}
