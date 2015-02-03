#![feature(core, slicing_syntax)]
extern crate irc;

use std::default::Default;
use irc::client::data::config::Config;
use irc::client::server::{IrcServer, Server};
use irc::client::server::utils::Wrapper;

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        port: Some(6697),
        use_ssl: Some(true),
        .. Default::default()
    };
    let irc_server = IrcServer::from_config(config).unwrap();
    // The wrapper provides us with methods like send_privmsg(...) and identify(...)
    let server = Wrapper::new(&irc_server);
    server.identify().unwrap();
    for message in server.iter() {
        let message = message.unwrap(); // We'll just panic if there's an error.
        print!("{}", message.into_string());
        if &message.command[] == "PRIVMSG" {
            if let Some(msg) = message.suffix {
                if msg.contains("pickles") {
                    server.send_privmsg(&message.args[0][], "Hi!").unwrap();
                }
            }
        }
    }
}
