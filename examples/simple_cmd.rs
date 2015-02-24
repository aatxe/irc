extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

// This is the same as simple.rs, except we use an Iterator over Commands
// instead of an Iterator over Messages. A Command is basically a parsed Message,
// so Commands and Messages are interchangeable. It is up to the library user to
// choose one.

fn main() {
    let config = Config {
        nickname: Some(format!("pickles")),
        alt_nicks: Some(vec![format!("bananas"), format!("apples")]),
        server: Some(format!("irc.fyrechat.net")),
        channels: Some(vec![format!("#vana")]),
        .. Default::default()
    };
    let irc_server = IrcServer::from_config(config).unwrap();
    // The wrapper provides us with methods like send_privmsg(...) and identify(...)
    let server = Wrapper::new(&irc_server);     
    server.identify().unwrap();
    for command in server.iter_cmd() {
        // Ignore errors
        // Use of unwrap() with iter_cmd() is discouraged because iter_cmd() is still unstable
        // and has trouble converting some custom Messages into Commands
        match command {
            Ok(cmd) => {
                print!("{}", cmd.to_message().into_string());
                match cmd {
                    Command::PRIVMSG(target, text) => {
                        if text[..].contains("pickles") {
                            server.send_privmsg(&target[..], "Hi!").unwrap();
                        }
                    },
                    _ => ()
                }
            },
            Err(_) => ()
        };
    }
}
