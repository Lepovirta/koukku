use std::fmt;
use std::str;
use hyper::header::{Header, Headers, HeaderFormat};
use hyper::error::{Result as HyperResult, Error as HyperError};

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

#[derive(Debug, Clone)]
pub struct HubSignature {
    pub digest: String,
    pub key: String,
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

        Ok(HubSignature {
            digest: parts[0].to_owned(),
            key: parts[1].to_owned(),
        })
    }
}

impl HeaderFormat for HubSignature {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(f.write_str(&self.digest));
        try!(f.write_str("="));
        f.write_str(&self.key)
    }
}

pub fn get_event(headers: &Headers) -> Result<&GithubEvent, Error> {
    get_header::<GithubEvent>(headers)
}

pub fn get_signature(headers: &Headers) -> Result<&HubSignature, Error> {
    get_header::<HubSignature>(headers)
}

fn get_header<H: Header + HeaderFormat>(headers: &Headers) -> Result<&H, Error> {
    headers.get::<H>().ok_or(Error::missing_header::<H>())
}
