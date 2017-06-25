extern crate futures;
extern crate irc;
extern crate tokio_core;

use std::default::Default;
use futures::future;
use irc::error;
use irc::client::prelude::*;
use tokio_core::reactor::Core;

fn main() {
    let cfg1 = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };
    let cfg2 = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.pdgn.co".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        use_ssl: Some(true),
        ..Default::default()
    };

    let configs = vec![cfg1, cfg2];

    // Create an event loop to run the multiple connections on.
    let mut reactor = Core::new().unwrap();
    let handle = reactor.handle();

    for config in configs {
        let server = IrcServer::from_config(config).unwrap();
        server.identify().unwrap();

        handle.spawn(server.stream().for_each(move |message| {
            process_msg(&server, message)
        }).map_err(|e| Err(e).unwrap()))
    }

    // You might instead want to join all the futures and run them directly.
    reactor.run(future::empty::<(), ()>()).unwrap();
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
