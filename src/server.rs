use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::io::Read;
use std::net::SocketAddr;
use hyper;
use hyper::uri::RequestUri;
use hyper::Server;
use hyper::server::{Request, Response, Listening, Handler};
use hyper::error::Result as HyperResult;
use openssl::crypto::hmac::hmac;
use openssl::crypto::memcmp;
use rustc_serialize::hex::ToHex;

use payload;
use conf::{Projects, Project};
use error::{Reason, Error, Result};
use header;
use header::{GithubEvent, HubSignature};

struct WebhookHandler {
    pub projects: Projects,
    pub send: Mutex<Sender<String>>,
}

impl WebhookHandler {
    fn new(projects: Projects, send: Sender<String>) -> WebhookHandler {
        WebhookHandler {
            projects: projects,
            send: Mutex::new(send),
        }
    }

    fn ping(&self) -> Result<String> {
        Ok("Pong".to_owned())
    }

    fn get_project(&self, repo: &str) -> Result<&Project> {
        self.projects
            .get(repo)
            .ok_or(Error::app(Reason::MissingProject, "No project found!"))
    }

    fn push(&self, readable: &mut Read, signature: &HubSignature) -> Result<String> {
        // Body
        let bytes = try!(read_bytes(readable));
        let json = try!(payload::bytes_to_json(&bytes));
        let repo = try!(payload::get_repo_name(&json));
        let branch = try!(payload::get_branch(&json));

        // Project
        let project = try!(self.get_project(repo));

        // Verify
        let _ = try!(verify(&signature, project.key.as_ref(), &bytes));
        if branch != project.branch {
            let error_msg = format!("Expected branch {} but got {}", project.branch, branch);
            return Err(Error::app(Reason::InvalidBranch, error_msg));
        }

        // Trigger
        let _ = try!(self.trigger_hook(repo));
        info!("Triggered hook for repo: {}", repo);

        Ok("Hook triggered".to_owned())
    }

    fn trigger_hook(&self, repo: &str) -> Result<()> {
        let s = try!(self.send.lock());
        let _ = try!(s.send(repo.to_owned()));
        Ok(())
    }
}

fn read_bytes(read: &mut Read) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let _ = try!(read.read_to_end(&mut buffer));
    Ok(buffer)
}

fn verify(signature: &HubSignature, key: &[u8], content: &[u8]) -> Result<()> {
    let result = hmac(signature.digest, key, content);
    let expected_hash: &[u8] = &signature.hash;

    if memcmp::eq(&result, &expected_hash) {
        Ok(())
    } else {
        let msg = format!("Verification failed. Expected hash {}, but received {}",
                          result.to_hex(),
                          expected_hash.to_hex());
        Err(Error::app(Reason::InvalidSignature, msg))
    }
}

impl Handler for WebhookHandler {
    fn handle(&self, mut req: Request, res: Response) {
        let remote_addr = &req.remote_addr.to_owned();
        let uri = &req.uri.to_owned();
        let result = match header::get_event(&req.headers) {
            Ok(GithubEvent::Ping) => self.ping(),
            Ok(GithubEvent::Push) => {
                header::get_signature(&req.headers).and_then(|sig| self.push(&mut req, &sig))
            }
            Err(err) => Err(err),
        };
        handle_result(result, res, remote_addr, uri);
    }
}

fn handle_result(result: Result<String>,
                 response: Response,
                 remote_addr: &SocketAddr,
                 uri: &RequestUri) {
    match result {
        Ok(contents) => send_bytes(response, &contents.into_bytes()),
        Err(err) => handle_error(err, response, remote_addr, uri),
    }
}

fn handle_error(err: Error, mut response: Response, remote_addr: &SocketAddr, uri: &RequestUri) {
    log_error(&err, remote_addr, uri);
    *response.status_mut() = hyper::BadRequest;
    send_bytes(response, b"Failed to trigger an update")
}

fn send_bytes(response: Response, bs: &[u8]) {
    if let Err(err) = response.send(bs) {
        error!("Failed to write response: {}", err);
    }
}

fn log_error(err: &Error, remote_addr: &SocketAddr, uri: &RequestUri) {
    error!("Failed request from {} to {}: {}", remote_addr, uri, err)
}

pub fn start(address: &str,
             threads: Option<usize>,
             projects: Projects,
             send: Sender<String>)
             -> HyperResult<Listening> {
    let server = try!(Server::http(address));
    let handler = WebhookHandler::new(projects, send);
    match threads {
        None => server.handle(handler),
        Some(t) => server.handle_threads(handler, t),
    }
}

#[cfg(test)]
mod tests {
    use super::WebhookHandler;
    use std::collections::HashMap;
    use std::sync::mpsc::{channel, Receiver};
    use std::io::Cursor;
    use std::fmt::Debug;
    use openssl::crypto::hash::Type;
    use rustc_serialize::hex::FromHex;
    use conf::Project;
    use header::HubSignature;
    use error::{Result, Reason, Error};

    const PAYLOAD: &'static str = "{ \"repository\": { \"full_name\": \"Lepovirta/koukku\" }, \
                                   \"ref\": \"ref/heads/master\" }";
    const HEX_SHA1: &'static str = "ddcfaf5fd20707cbb5aae68b0cf0904be7de1b7f";

    const INVALID_PAYLOAD: &'static str = "{ \"repository\": { \"something\": \
                                           \"Lepovirta/koukku\" } }";
    const INVALID_HEX_SHA1: &'static str = "364fc28fcf2f50fe5760e7e09e4c5efff04115d4";

    const INVALID_BRANCH: &'static str = "{ \"repository\": { \"full_name\": \"Lepovirta/koukku\" \
                                          }, \"ref\": \"ref/heads/other\" }";
    const INVALID_BRANCH_HEX_SHA1: &'static str = "4b673629d7f6203cc7636d12416808b8b9348146";

    const UNKNOWN_REPO: &'static str = "{ \"repository\": { \"full_name\": \"Lepovirta/lepo\" }, \
                                        \"ref\": \"ref/heads/master\" }";
    const KEY: &'static str = "foobar";
    const REPO: &'static str = "Lepovirta/koukku";

    fn setup() -> (WebhookHandler, Receiver<String>) {
        let (tx, rx) = channel();
        let mut m = HashMap::new();
        let project = Project {
            id: "koukku".to_owned(),
            repo: REPO.to_owned(),
            branch: "master".to_owned(),
            key: KEY.to_owned(),
            command: "dostuff.sh".to_owned(),
        };
        m.insert(project.repo.to_owned(), project);
        (WebhookHandler::new(m, tx), rx)
    }

    fn sha1sig(sha1str: &str) -> HubSignature {
        let sha1 = sha1str.from_hex().unwrap();
        HubSignature {
            digest: Type::SHA1,
            hash: sha1,
        }
    }

    fn cursor_from_str(contents: &str) -> Cursor<Vec<u8>> {
        let payload: Vec<u8> = contents.to_owned().into();
        Cursor::new(payload)
    }

    fn assert_reason<T>(result: &Result<T>, expected_reason: Reason)
        where T: Debug
    {
        match *result {
            Ok(ref v) => panic!("Expected a failed result, but got success: {:?}", v),
            Err(Error::App(ref reason, _)) => assert_eq!(reason, &expected_reason),
            Err(ref e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn correct_everything() {
        let mut cursor = cursor_from_str(PAYLOAD);
        let sig = sha1sig(HEX_SHA1);
        let (handler, rx) = setup();

        let result = handler.push(&mut cursor, &sig);

        assert!(result.is_ok(), "result = {:?}", result);
        assert_eq!(rx.recv().unwrap(), REPO);
    }

    #[test]
    fn incorrect_payload() {
        let mut cursor = cursor_from_str(INVALID_PAYLOAD);
        let sig = sha1sig(HEX_SHA1);
        let (handler, rx) = setup();

        let result = handler.push(&mut cursor, &sig);

        assert_reason(&result, Reason::MissingFields);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn incorrect_signature() {
        let mut cursor = cursor_from_str(PAYLOAD);
        let sig = sha1sig(INVALID_HEX_SHA1);
        let (handler, rx) = setup();

        let result = handler.push(&mut cursor, &sig);

        assert_reason(&result, Reason::InvalidSignature);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn incorrect_repository() {
        let mut cursor = cursor_from_str(UNKNOWN_REPO);
        let sig = sha1sig(HEX_SHA1);
        let (handler, rx) = setup();

        let result = handler.push(&mut cursor, &sig);

        assert_reason(&result, Reason::MissingProject);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn incorrect_branch() {
        let mut cursor = cursor_from_str(INVALID_BRANCH);
        let sig = sha1sig(INVALID_BRANCH_HEX_SHA1);
        let (handler, rx) = setup();

        let result = handler.push(&mut cursor, &sig);

        assert_reason(&result, Reason::InvalidBranch);
        assert!(rx.try_recv().is_err());
    }
}
