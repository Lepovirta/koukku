use std::sync::{Mutex, Arc};
use std::sync::mpsc::Sender;
use std::io::Read;
use std::net::SocketAddr;
use hyper;
use hyper::uri::RequestUri;
use hyper::Server;
use hyper::server::{Request, Response, Listening, Handler};
use hyper::error::Result as HyperResult;
use openssl::crypto::hmac::hmac;
use rustc_serialize::hex::ToHex;

use payload;
use conf::Conf;
use error::{Error, Result};
use header;
use header::{GithubEvent, HubSignature};

struct WebhookHandler {
    conf: Arc<Conf>,
    send: Mutex<Sender<String>>,
}

impl WebhookHandler {
    fn new(conf: Arc<Conf>, send: Sender<String>) -> WebhookHandler {
        WebhookHandler {
            conf: conf,
            send: Mutex::new(send),
        }
    }

    fn ping(&self) -> Result<String> {
        Ok("Pong".to_owned())
    }

    fn push(&self, mut req: Request) -> Result<String> {
        // Headers
        let signature = try!(header::get_signature(&req.headers));

        // Body
        let bytes = try!(read_bytes(&mut req));
        let json = try!(payload::bytes_to_json(&bytes));
        let repo = try!(payload::get_repo_name(&json));

        // Conf
        let key = try!(get_key(&self.conf, repo));

        // Verify
        let _ = try!(verify(&signature, key, &bytes));

        // Trigger
        let _ = try!(self.trigger_hook(repo));
        info!("Triggered hook for repo: {}", repo);

        Ok("Hook triggered".to_owned())
    }

    fn trigger_hook(&self, repo: &str) -> Result<()> {
        let s = try!(self.send.lock().map_err(|_| Error::from("Failed to lock send")));
        s.send(repo.to_owned()).map_err(|_| Error::from("Failed to send trigger"))
    }
}

fn get_key<'a>(conf: &'a Conf, repo: &str) -> Result<&'a [u8]> {
    conf.get_project(repo)
        .map(|project| project.key.as_ref())
        .ok_or(Error::from("No key found!"))
}

fn read_bytes(read: &mut Read) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let _ = try!(read.read_to_end(&mut buffer));
    Ok(buffer)
}

fn verify(signature: &HubSignature, key: &[u8], content: &[u8]) -> Result<()> {
    let result = hmac(signature.digest, key, content);
    let expected_hash: &[u8] = &signature.hash;

    if result == expected_hash {
        // TODO: use constant time comparison to avoid timing attacks
        Ok(())
    } else {
        let msg = format!("Verification failed. Received expected hash {}, but received {}",
                          result.to_hex(),
                          expected_hash.to_hex());
        Err(Error::from(msg))
    }
}

impl Handler for WebhookHandler {
    fn handle(&self, req: Request, res: Response) {
        let remote_addr = &req.remote_addr.to_owned();
        let uri = &req.uri.to_owned();
        let result = match header::get_event(&req.headers) {
            Ok(GithubEvent::Ping) => self.ping(),
            Ok(GithubEvent::Push) => self.push(req),
            Err(err) => Err(err),
        };
        handle_result(result, res, remote_addr, uri);
    }
}

fn handle_result(result: Result<String>,
                 response: Response,
                 remote_addr: &SocketAddr, uri: &RequestUri) {
    match result {
        Ok(contents) => send_bytes(response, &contents.into_bytes()),
        Err(err) => handle_error(err, response, remote_addr, uri),
    }
}

fn handle_error(err: Error, mut response: Response,
                remote_addr: &SocketAddr, uri: &RequestUri) {
    log_error(&err, remote_addr, uri);
    *response.status_mut() = hyper::BadRequest;
    send_bytes(response, b"Invalid response")
}

fn send_bytes(response: Response, bs: &[u8]) {
    if let Err(err) = response.send(bs) {
        error!("Failed to write response: {}", err);
    }
}

fn log_error(err: &Error, remote_addr: &SocketAddr, uri: &RequestUri) {
    error!("Failed request from {} to {}: {}", remote_addr, uri, err)
}

pub fn start(address: &str, conf: Arc<Conf>, send: Sender<String>) -> HyperResult<Listening> {
    let server = try!(Server::http(address));
    let handler = WebhookHandler::new(conf, send);
    server.handle(handler)
}
