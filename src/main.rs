extern crate ini;
extern crate crypto;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;

mod error;
mod header;
mod server;
mod conf;

use clap::{Arg, App};

fn main() {
    let matches = App::new("hubikoukku")
        .version("0.1")
        .author("jkpl")
        .about("Github Webhook server")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Configuration file location")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("server")
             .short("s")
             .long("server")
             .value_name("HOST:PORT")
             .help("The address and port to run the server on")
             .takes_value(true)
             .required(true))
        .get_matches();

    let config = matches.value_of("config").unwrap();
    let server = matches.value_of("server").unwrap();
    start(&config, &server);
}

fn start(config: &str, server: &str) {
    let _ = env_logger::init().unwrap();
    let s = conf::Conf::from_file(config).unwrap();
    println!("{}", &s);
    info!("Starting hubikoukku server");
    let _ = server::start(server).unwrap();
}
