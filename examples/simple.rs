#![feature(if_let)]
#![feature(slicing_syntax)]
extern crate irc;

use std::default::Default;
use irc::data::config::Config;
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        alt_nicks: Some(vec![format!("bananas"), format!("apples")]),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        .. Default::default()
    };
    let irc_server = IrcServer::from_config(config).unwrap();
    // The wrapper provides us with methods like send_privmsg(...) and identify(...)
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
