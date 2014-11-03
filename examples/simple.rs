#![feature(if_let)]
#![feature(slicing_syntax)]
extern crate irc;

use std::collections::HashMap;
use irc::data::config::Config;
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

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
    let irc_server = IrcServer::from_config(config).unwrap();
    let server = Wrapper::new(&irc_server);
    server.identify().unwrap();
    for message in server.iter() {
        print!("{}", message.into_string());
        if message.command[] == "PRIVMSG" {
            if let Some(msg) = message.suffix {
                if msg.contains("pickles") {
                    server.send_privmsg(message.args[0][], "Hi!").unwrap();
                }
            }
        }
    }
}
