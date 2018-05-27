extern crate irc;

use std::default::Default;
use std::thread;
use std::time::Duration;
use irc::client::prelude::*;

// NOTE: you can find an asynchronous version of this example with `IrcReactor` in `tooter.rs`.
fn main() {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };
    let client = IrcClient::from_config(config).unwrap();
    client.identify().unwrap();
    let client2 = client.clone();
    // Let's set up a loop that just prints the messages.
    thread::spawn(move || {
        client2.stream().map(|m| print!("{}", m)).wait().count();
    });
    loop {
        client.send_privmsg("#irc-crate", "TWEET TWEET").unwrap();
        thread::sleep(Duration::new(10, 0));
    }
}
