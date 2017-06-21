extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        port: Some(6697),
        use_ssl: Some(true),
        ..Default::default()
    };
    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
    server.stream().for_each(|message| {
        print!("{}", message);
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                if msg.contains("pickles") {
                    server.send_privmsg(target, "Hi!").unwrap();
                }
            }
            _ => (),
        }
        Ok(())
    }).wait().unwrap()
}
