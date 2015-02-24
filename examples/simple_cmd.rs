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
    for command in server.iter_cmd() {
        // Use of unwrap() on the results of iter_cmd() is currently discouraged on servers where
        // the v3 capabilities extension is enabled, and the current lapse in specification 
        // compliance on that specific command will then cause the program to panic.
        if let Ok(Command::PRIVMSG(chan, msg)) = command { // Ignore errors.
            if msg.contains("pickles") {
                server.send_privmsg(&chan, "Hi!").unwrap();
            }
        }
    }
}
