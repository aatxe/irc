#![feature(slicing_syntax)]
extern crate irc;

use std::collections::HashMap;
use irc::Server;
use irc::bot::IrcServer;
use irc::data::{Config, Message};

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
    let mut server = IrcServer::new_with_config(config).unwrap();
    server.send(Message::new(None, "NICK", vec!["pickles"], None)).unwrap();
    server.send(Message::new(None, "USER", vec!["pickles", "0", "*", "pickles"], None)).unwrap();
    for message in server {
        println!("RCV: {}", message);
    }
}
