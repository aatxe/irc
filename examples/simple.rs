extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        alt_nicks: Some(vec![format!("bananas"), format!("apples")]),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        .. Default::default()
    };
    let server = IrcServer::from_config(config).unwrap();   
    server.identify().unwrap();
    for message in server.iter() {
        let message = message.unwrap(); // We'll just panic if there's an error.
        print!("{}", message.into_string());
        if &message.command[..] == "PRIVMSG" {
            if let Some(msg) = message.suffix {
                if msg.contains("pickles") {
                    server.send_privmsg(&message.args[0], "Hi!").unwrap();
                }
            }
        }
    }
}
