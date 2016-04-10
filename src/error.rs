use std::{fmt, result, io};
use std::error::Error as StdError;
use std::str::Utf8Error;
use std::sync::mpsc::{SendError, RecvError};
use std::sync::PoisonError;
use hyper::error::Error as HyperError;
use ini::ini::Error as IniError;
use rustc_serialize::hex::FromHexError;
use serde_json::error::Error as JsonError;

use self::Error::{Hyper, App, Utf8, Io, Ini, Hex, Json, Mutex, Channel};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Reason {
    InvalidConf,
    InvalidSignature,
    InvalidRepository,
    InvalidBranch,
    InvalidPath,
    MissingHeader,
    MissingFields,
    MissingProject,
    CommandFailed,
}

#[derive(Debug)]
pub enum Error {
    App(Reason, String),
    Ini(String),
    Mutex(String),
    Channel(String),
    Hyper(HyperError),
    Utf8(Utf8Error),
    Io(io::Error),
    Hex(FromHexError),
    Json(JsonError),
}

impl Error {
    pub fn app<S: Into<String>>(reason: Reason, desc: S) -> Error {
        App(reason, desc.into())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            App(_, ref s) => &s,
            Ini(ref s) => &s,
            Mutex(ref s) => &s,
            Channel(ref s) => &s,
            Hyper(ref err) => err.description(),
            Utf8(ref err) => err.description(),
            Io(ref err) => err.description(),
            Hex(ref err) => err.description(),
            Json(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Hyper(ref err) => Some(err),
            Utf8(ref err) => Some(err),
            Io(ref err) => Some(err),
            Json(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Hyper(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Io(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Error {
        Utf8(err)
    }
}

impl From<IniError> for Error {
    fn from(err: IniError) -> Error {
        Ini(format!("{}", err))
    }
}

impl From<FromHexError> for Error {
    fn from(err: FromHexError) -> Error {
        Hex(err)
    }
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Error {
        Json(err)
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Error {
        Mutex(format!("{}", err))
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(err: SendError<T>) -> Error {
        Channel(format!("{}", err))
    }
}

impl From<RecvError> for Error {
    fn from(err: RecvError) -> Error {
        Channel(err.description().into())
    }
}
