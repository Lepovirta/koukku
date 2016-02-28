use std::{fmt, result, io};
use std::error::Error as StdError;
use std::str::Utf8Error;
use hyper::error::Error as HyperError;
use ini::ini::Error as IniError;
use rustc_serialize::hex::FromHexError;
use serde_json::error::Error as JsonError;

use self::Error::{Hyper, Generic, Utf8, Io, Ini, Hex, Json};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Ini(String),
    Hyper(HyperError),
    Generic(String),
    Utf8(Utf8Error),
    Io(io::Error),
    Hex(FromHexError),
    Json(JsonError),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Generic(ref s) => &s,
            Ini(ref s) => &s,
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

impl From<String> for Error {
    fn from(s: String) -> Error {
        Generic(s)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(s: &'a str) -> Error {
        Generic(s.to_owned())
    }
}
