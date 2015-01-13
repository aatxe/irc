#![allow(unstable)]
#![feature(slicing_syntax)]
extern crate irc;

use std::default::Default;
use std::sync::Arc;
use std::thread::Thread;
use irc::client::data::{Command, Config};
use irc::client::server::{IrcServer, Server};
use irc::client::server::utils::Wrapper;

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        .. Default::default()
    };
    let irc_server = Arc::new(IrcServer::from_config(config).unwrap());
    irc_server.conn().set_keepalive(Some(5)).unwrap();
    // The wrapper provides us with methods like send_privmsg(...) and identify(...)
    let _ = Thread::scoped(move || { 
        let server = Wrapper::new(&*irc_server);
        server.identify().unwrap();
        loop {
            let mut quit = false;
            for msg in server.iter() {
                match msg {
                    Ok(msg) => { 
                        print!("{}", msg.into_string());
                        match Command::from_message(&msg) {
                            Ok(Command::PRIVMSG(_, msg)) => if msg.contains("bye") { 
                                server.send_quit("").unwrap() 
                            },
                            Ok(Command::ERROR(msg)) if msg.contains("Quit") => quit = true,
                            _ => (),
                        }
                    },
                    Err(_)  => break,
                }
            }
            if quit { break }
            irc_server.reconnect().unwrap();
            server.identify().unwrap();
        }
    }).join(); 
}
