use failure::{Backtrace, Context, Fail};
use std::{fmt, result};

/// A type alias for handling errors throughout crossref.
pub type Result<T> = result::Result<T, Error>;

/// An error that can occur while interacting while handling rustika.
#[derive(Debug)]
pub struct Error {
    ctx: Context<ErrorKind>,
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

    /// an error that occurred while operating with [reqwest]
    #[fail(display = "{}", reqwest)]
    ReqWest {
        /// the notification
        reqwest: reqwest::Error,
    },
    /// if a error in serde occurred
    #[fail(display = "invalid serde: {}", error)]
    Serde { error: serde_json::Error },
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
