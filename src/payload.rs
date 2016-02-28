use std::str;
use std::io::Read;
use serde_json;
use serde_json::Value as JsonValue;

use error::{Error, Result};

pub fn get_repo_name(json: &JsonValue) -> Result<&str> {
    json.lookup("repository.full_name")
        .and_then(|v| v.as_string())
        .ok_or(Error::from("No repository name found"))
}

pub fn bytes_to_json(bytes: &[u8]) -> Result<JsonValue> {
    let s = try!(str::from_utf8(bytes).map_err(Error::from));
    str_to_json(s)
}

pub fn read_json(reader: &mut Read) -> Result<JsonValue> {
    let mut buffer = String::new();
    let _ = try!(reader.read_to_string(&mut buffer).map_err(Error::from));
    str_to_json(&buffer)
}

pub fn str_to_json(s: &str) -> Result<JsonValue> {
    serde_json::from_str(s).map_err(Error::from)
}
