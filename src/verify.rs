use std::collections::HashMap;
use openssl::crypto::hash::Type;
use openssl::crypto::hmac::hmac;

use error::{Result, Error};

pub fn verify(digest_name: &str, key: &[u8], hash: &[u8], content: &[u8]) -> Result<()> {
    let digest = try!(str_to_digest(digest_name));
    let result = hmac(digest, key, content);
    if result == hash { // TODO: use constant time comparison to avoid timing attacks
        Ok(())
    } else {
        Err(Error::from("Verification failed"))
    }
}

fn str_to_digest(digest_name: &str) -> Result<Type> {
    match digest_name {
        "sha1" => Ok(Type::SHA1),
        _ => Err(no_such_digest(digest_name)),
    }
}

fn no_such_digest(digest: &str) -> Error {
    Error::from(format!("No such digest {}", digest))
}


#[cfg(test)]
mod test {
    use rustc_serialize::hex::FromHex;
    use super::verify;

    const DIGEST_NAME: &'static str = "sha1";
    const KEY: &'static [u8] = b"thisdasecretyo";
    const HASH_STR: &'static str = "d68330e54f6ad125813d6331e4037fdbe35c0895";
    const CONTENT: &'static [u8] = b"foobar";

    fn hash() -> Vec<u8> {
        HASH_STR.from_hex().unwrap()
    }

    #[test]
    fn sha1() {
        let result = verify(DIGEST_NAME, KEY, &hash(), CONTENT);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn unknown_digest() {
        let result = verify("wat", KEY, &hash(), CONTENT);
        assert_eq!(result.is_ok(), false);
    }


    #[test]
    fn wrong_content() {
        let result = verify(DIGEST_NAME, KEY, &hash(), b"justfoo");
        assert_eq!(result.is_ok(), false);
    }
}
