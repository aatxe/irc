extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        alt_nicks: Some(vec!["bananas".to_owned(), "apples".to_owned()]),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };

    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();

    server.for_each_incoming(|message| {
        print!("{}", message);
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                if msg.contains("pickles") {
                    server.send_privmsg(target, "Hi!").unwrap();
                }
            }
            _ => (),
        }
    }).unwrap()
}
