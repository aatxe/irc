extern crate irc;

use std::default::Default;
use std::sync::Arc;
use std::thread::spawn;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        .. Default::default()
    };
    let server = Arc::new(IrcServer::from_config(config).unwrap());
    // FIXME: if set_keepalive is stabilized, this can be readded.
    // server.conn().set_keepalive(Some(5)).unwrap();
    let server = server.clone();
    let _ = spawn(move || {
        server.identify().unwrap();
        loop {
            let mut quit = false;
            for msg in server.iter() {
                match msg {
                    Ok(msg) => {
                        print!("{}", msg.into_string());
                        match (&msg).into() {
                            Ok(Command::PRIVMSG(_, msg)) => if msg.contains("bye") {
                                server.send_quit("").unwrap()
                            },
                            Ok(Command::ERROR(ref msg)) if msg.contains("Quit") => quit = true,
                            _ => (),
                        }
                    },
                    Err(_)  => break,
                }
            }
            if quit { break }
            server.reconnect().unwrap();
            server.identify().unwrap();
        }
    }).join();
}
