use std::collections::HashMap;
use crypto::digest::Digest;
use crypto::mac::{Mac, MacResult};
use crypto::hmac::Hmac;
use crypto::sha1::Sha1;

use error::{Result, Error};

pub fn verify(digest_name: &str, key: &[u8], hash: &[u8], content: &[u8]) -> Result<()> {
    let mut hmac = match digest_name {
        "sha1" => Hmac::new(Sha1::new(), key),
        _ => return Err(no_such_digest(digest_name)),
    };
    verify_hmac(&mut hmac, hash, content)
}

fn verify_hmac<D: Digest>(hmac: &mut Hmac<D>, hash: &[u8], content: &[u8]) -> Result<()> {
    hmac.input(content);
    let result = hmac.result();
    match MacResult::new(hash) == result {
        true => Ok(()),
        false => Err(Error::from("Verification failed")),
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
