extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

// This example is meant to be a direct analogue to simple.rs using the reactor API.
fn main() {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        alt_nicks: Some(vec!["bananas".to_owned(), "apples".to_owned()]),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, |client, message| {
        print!("{}", message);
        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if msg.contains("pickles") {
                client.send_privmsg(target, "Hi!")?;
            }
        }
        Ok(())
    });

    reactor.run().unwrap();
}
