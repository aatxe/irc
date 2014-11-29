#![feature(if_let, slicing_syntax)]
extern crate irc;

use std::collections::HashMap;
use std::io::timer::sleep;
use std::sync::Arc;
use std::time::duration::Duration;
use irc::data::config::Config;
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

fn main() {
    let config = Config {
        owners: vec!("awe".into_string()),
        nickname: "tweeter".into_string(),
        username: "tweeter".into_string(),
        realname: "tweeter".into_string(),
        password: "".into_string(),
        server: "irc.fyrechat.net".into_string(),
        port: 6667,
        use_ssl: false,
        channels: vec!("#vana".into_string()),
        options: HashMap::new(),
    };
    let irc_server = Arc::new(IrcServer::from_config_with_timeout(config, 10 * 1000).unwrap());
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
    // Even though sending and reading both block, this will still be sent every ten seconds
    // thanks to the timeout we have set for the server.
    loop {
        server.send_privmsg("#vana", "TWEET TWEET").unwrap();
        sleep(Duration::seconds(10))
    }
}

