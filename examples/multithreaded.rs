extern crate irc;

use std::default::Default;
use std::thread::spawn;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#vana".to_owned()]),
        .. Default::default()
    };
    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();
    let server = server.clone();
    let _ = spawn(move || {
        for msg in server.iter() {
            print!("{}", msg.unwrap());
        }
    }).join(); // You might not want to join here for actual multi-threading.
}
