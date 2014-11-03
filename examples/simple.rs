#![feature(slicing_syntax)]
extern crate irc;

use std::collections::HashMap;
use irc::data::config::Config;
use irc::server::{IrcServer, Server};
use irc::server::utils::identify;

fn main() {
    let config = Config {
        owners: vec!("awe".into_string()),
        nickname: "pickles".into_string(),
        username: "pickles".into_string(),
        realname: "pickles".into_string(),
        password: "".into_string(),
        server: "irc.fyrechat.net".into_string(),
        port: 6667,
        channels: vec!("#vana".into_string()),
        options: HashMap::new(),
    };
    let server = IrcServer::from_config(config).unwrap();
    identify(&server).unwrap();
    for message in server.iter() {
        println!("{}", message);
    }
}
