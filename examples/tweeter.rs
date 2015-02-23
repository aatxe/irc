#![feature(old_io, std_misc)]
extern crate irc;

use std::default::Default;
use std::old_io::timer::sleep;
use std::sync::Arc;
use std::thread::spawn;
use std::time::duration::Duration;
use irc::client::prelude::*;

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
    let server = Wrapper::new(&*irc_server2);
    server.identify().unwrap();
    // Let's set up a loop that just prints the messages.
    spawn(move || { 
        irc_server.iter().map(|m| print!("{}", m.unwrap().into_string())).count(); 
    });
    loop {
        server.send_privmsg("#vana", "TWEET TWEET").unwrap();
        sleep(Duration::seconds(10))
    }
}
