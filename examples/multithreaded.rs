#![feature(if_let)]
#![feature(slicing_syntax)]
extern crate irc;

use std::collections::HashMap;
use std::sync::Arc;
use irc::data::config::Config;
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

fn main() {
    let config = config();
    let irc_server = Arc::new(IrcServer::from_config(config).unwrap());
    // The wrapper provides us with methods like send_privmsg(...) and identify(...)
    let server = Wrapper::new(&*irc_server);
    server.identify().unwrap();
    let server = irc_server.clone();
    // We won't use a wrapper here because we don't need the added functionality.
    spawn(proc() { 
        for msg in server.iter() {
            print!("{}", msg.into_string());
        }
    });
}

#[cfg(feature = "encode")]
fn config() -> Config {
    Config {
        owners: vec!("awe".into_string()),
        nickname: "pickles".into_string(),
        username: "pickles".into_string(),
        realname: "pickles".into_string(),
        password: "".into_string(),
        server: "irc.fyrechat.net".into_string(),
        port: 6667,
        use_ssl: false,
        encoding: format!("UTF-8"),
        channels: vec!("#vana".into_string()),
        options: HashMap::new(),
    }
}

#[cfg(not(feature = "encode"))]
fn config() -> Config {
    Config {
        owners: vec!("awe".into_string()),
        nickname: "pickles".into_string(),
        username: "pickles".into_string(),
        realname: "pickles".into_string(),
        password: "".into_string(),
        server: "irc.fyrechat.net".into_string(),
        port: 6667,
        use_ssl: false,
        channels: vec!("#vana".into_string()),
        options: HashMap::new(),
    }
}
