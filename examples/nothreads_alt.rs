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
        nickname: Some("pickles1".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };

    let cfg2 = Config {
        nickname: Some("pickles2".to_owned()),
        server: Some("irc.fyrechat.net".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        ..Default::default()
    };

    let configs = vec![cfg1, cfg2];

    let (futures, mut reactor) = configs.iter().fold(
        (vec![], Core::new().unwrap()),
        |(mut acc, mut reactor), config| {
            let handle = reactor.handle();
            // First, we run the future representing the connection to the server.
            // After this is complete, we have connected and can send and receive messages.
            let server = reactor.run(IrcServer::new_future(handle, config).unwrap()).unwrap();
            server.identify().unwrap();

            // Add the future for processing messages from the current server to the accumulator.
            acc.push(server.stream().for_each(move |message| {
                process_msg(&server, message)
            }));

            // We then thread through the updated accumulator and the reactor.
            (acc, reactor)
        }
    );

    // Here, we join on all of the futures representing the message handling for each server.
    reactor.run(future::join_all(futures)).unwrap();
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
