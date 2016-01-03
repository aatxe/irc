extern crate irc;

use std::default::Default;
use std::thread::{sleep, spawn};
use std::time::Duration;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        .. Default::default()
    };
    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
    let server2 = server.clone();
    // Let's set up a loop that just prints the messages.
    spawn(move || {
        server2.iter().map(|m| print!("{}", m.unwrap().into_string())).count();
    });
    loop {
        server.send_privmsg("#vana", "TWEET TWEET").unwrap();
        sleep(Duration::new(10, 0));
    }
}
