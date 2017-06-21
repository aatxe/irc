extern crate irc;

use std::default::Default;
use std::thread;
use std::time::Duration;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };
    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
    let server2 = server.clone();
    // Let's set up a loop that just prints the messages.
    thread::spawn(move || {
        server2.stream().map(|m| print!("{}", m)).wait().count();
    });
    loop {
        server.send_privmsg("#irc-crate", "TWEET TWEET").unwrap();
        thread::sleep(Duration::new(10, 0));
    }
}
