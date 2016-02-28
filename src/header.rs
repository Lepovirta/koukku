use std::fmt::{self, Debug};
use std::str;
use hyper::header::{Header, Headers, HeaderFormat};
use hyper::error::{Result as HyperResult, Error as HyperError};
use openssl::crypto::hash::Type;

use error::Error;

// We only care about pings and pushes
#[derive(Debug, Clone, Copy)]
pub enum GithubEvent {
    Ping,
    Push,
}

impl Header for GithubEvent {
    fn header_name() -> &'static str {
        "X-Github-Event"
    }

    fn parse_header(raw: &[Vec<u8>]) -> HyperResult<GithubEvent> {
        if raw.len() == 1 {
            let line = &raw[0];
            match str::from_utf8(line) {
                Ok("ping") => return Ok(GithubEvent::Ping),
                Ok("push") => return Ok(GithubEvent::Push),
                _ => (),
            }
        }
        Err(HyperError::Header)
    }
}

impl HeaderFormat for GithubEvent {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GithubEvent::Ping => f.write_str("ping"),
            GithubEvent::Push => f.write_str("push"),
        }
    }
}

#[derive(Clone)]
pub struct HubSignature {
    pub digest: Type,
    pub hash: String,
}

impl Header for HubSignature {
    fn header_name() -> &'static str {
        "X-Hub-Signature"
    }

    fn parse_header(raw: &[Vec<u8>]) -> HyperResult<HubSignature> {
        if raw.len() != 1 {
            return Err(HyperError::Header);
        }

        let line_str = try!(str::from_utf8(&raw[0]).map_err(|_| HyperError::Header));

        let parts: Vec<&str> = line_str.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(HyperError::Header);
        }

        let digest = try!(str_to_digest(parts[0]));

        Ok(HubSignature {
            digest: digest,
            hash: parts[1].to_owned(),
        })
    }
}

fn str_to_digest(digest_name: &str) -> HyperResult<Type> {
    match digest_name {
        "sha1" => Ok(Type::SHA1),
        _ => Err(HyperError::Header),
    }
}

impl Debug for HubSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(format_digest(f, &self.digest));
        try!(f.write_str("="));
        f.write_str(&self.hash)
    }
}

impl HeaderFormat for HubSignature {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f)
    }
}

fn format_digest(f: &mut fmt::Formatter, digest: &Type) -> fmt::Result {
    match *digest {
        Type::SHA1 => f.write_str("sha1"),
        _ => Err(fmt::Error),
    }
}

pub fn get_event(headers: &Headers) -> Result<GithubEvent, Error> {
    get_header::<GithubEvent>(headers).map(|h| h.to_owned())
}

pub fn get_signature(headers: &Headers) -> Result<HubSignature, Error> {
    get_header::<HubSignature>(headers).map(|h| h.to_owned())
}

fn get_header<H: Header + HeaderFormat>(headers: &Headers) -> Result<&H, Error> {
    headers.get::<H>().ok_or(missing_header::<H>())
}

fn missing_header<H: Header>() -> Error {
    Error::Generic("Missing header ".to_string() + H::header_name())
}
