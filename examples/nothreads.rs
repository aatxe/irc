extern crate irc;

use std::default::Default;
use irc::error;
use irc::client::prelude::*;

fn main() {
    let cfg1 = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };

    let cfg2 = Config {
        nickname: Some("bananas".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };

    let configs = vec![cfg1, cfg2];

    let mut reactor = IrcReactor::new().unwrap();

    for config in configs {
        // Immediate errors like failure to resolve the server's name or to establish any connection will
        // manifest here in the result of prepare_server_and_connect.
        let server = reactor.prepare_server_and_connect(&config).unwrap();
        server.identify().unwrap();
        // Here, we tell the reactor to setup this server for future handling (in run) using the specified
        // handler function process_msg.
        reactor.register_server_with_handler(server, process_msg);
    }

    // Runtime errors like a dropped connection will manifest here in the result of run.
    reactor.run().unwrap();
}

fn process_msg(server: &IrcServer, message: Message) -> error::Result<()> {
    print!("{}", message);
    match message.command {
        Command::PRIVMSG(ref target, ref msg) => {
            if msg.contains("pickles") {
                server.send_privmsg(target, "Hi!")?;
            }
        }
        _ => (),
    }
    Ok(())
}
