use std::str;
use serde_json;
use serde_json::Value as JsonValue;

use error::{Reason, Error, Result};

pub fn get_repo_name(json: &JsonValue) -> Result<&str> {
    json.lookup("repository.full_name")
        .and_then(|v| v.as_string())
        .ok_or(Error::app(Reason::MissingFields, "No repository name found"))
}

pub fn get_branch(json: &JsonValue) -> Result<&str> {
    json.lookup("ref")
        .and_then(|v| v.as_string())
        .and_then(|v| v.splitn(3, '/').nth(2)) // rev/heads/branch
        .ok_or(Error::app(Reason::MissingFields, "No branch found"))
}

pub fn bytes_to_json(bytes: &[u8]) -> Result<JsonValue> {
    let s = try!(str::from_utf8(bytes));
    str_to_json(s)
}

pub fn str_to_json(s: &str) -> Result<JsonValue> {
    serde_json::from_str(s).map_err(Error::from)
}
