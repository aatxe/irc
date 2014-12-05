#![feature(if_let, slicing_syntax)]
extern crate irc;

use std::default::Default;
use std::io::timer::sleep;
use std::sync::Arc;
use std::time::duration::Duration;
use irc::data::config::Config;
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        .. Default::default()
    }; 
    let irc_server = Arc::new(IrcServer::from_config(config).unwrap());
    let irc_server2 = irc_server.clone();
    // The wrapper provides us with methods like send_privmsg(...) and identify(...)
    let server = Wrapper::new(&*irc_server);
    server.identify().unwrap();
    // Let's set up a loop that ignores timeouts, and reads perpetually.
    // n.b. this shouldn't exit automatically if the connection closes. 
    spawn(proc() { 
        let mut iter = irc_server2.iter();
        loop {
            if let Some(msg) = iter.next() {
                print!("{}", msg.into_string());
            }
        }
    });
    loop {
        server.send_privmsg("#vana", "TWEET TWEET").unwrap();
        sleep(Duration::seconds(10))
    }
}
