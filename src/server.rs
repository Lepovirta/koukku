use std::error::Error as StdError;
use hyper;
use hyper::Server;
use hyper::server::{Request, Response, Listening, Handler};
use hyper::error::Result as HyperResult;

use error::{Error, Result};
use header;
use header::{GithubEvent, HubSignature};


struct WebhookHandler;

impl WebhookHandler {
    fn pong(&self, res: Response) -> Result<()> {
        try!(res.send(b"Pong").map_err(Error::from));
        Ok(())
    }

    fn push(&self, req: Request, res: Response) -> Result<()> {
        info!("Got push event");
        Ok(())
    }

    fn error(&self, err: Error, req: Request, mut res: Response) -> Result<()> {
        info!("Failed request from {} to {}: {}",
              req.remote_addr,
              req.uri,
              err);
        *res.status_mut() = hyper::BadRequest;
        Ok(())
    }
}

impl Handler for WebhookHandler {
    fn handle(&self, req: Request, mut res: Response) {
        let result = match header::get_event(&req.headers) {
            Ok(&GithubEvent::Ping) => self.pong(res),
            Ok(&GithubEvent::Push) => self.push(req, res),
            Err(err) => self.error(err, req, res),
        };
        log_result(&result);
    }
}

fn log_result<O>(result: &Result<O>) {
    match *result {
        Err(ref err) => {
            error!("Error occurred while executing handler: {}",
                   err.description())
        }
        _ => (),
    }
}

pub fn start(address: &str) -> HyperResult<Listening> {
    let server = try!(Server::http(address));
    server.handle(WebhookHandler)
}
