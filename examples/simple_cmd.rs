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
        // Use of unwrap() on the results of iter_cmd() is discouraged since response codes will be
        // received as parsing errors when using this type of iterator.
        if let Ok(Command::PRIVMSG(chan, msg)) = command { // Ignore errors.
            if msg.contains("pickles") {
                server.send_privmsg(&chan, "Hi!").unwrap();
            }
        }
    }
}
