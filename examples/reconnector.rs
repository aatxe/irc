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

    loop {
        let res = configs.iter().fold(Ok(()), |acc, config| {
            acc.and(
                reactor.prepare_client_and_connect(config.clone()).and_then(|client| {
                    client.identify().and(Ok(client))
                }).and_then(|client| {
                    reactor.register_client_with_handler(client, process_msg);
                    Ok(())
                })
            )
        }).and_then(|()| reactor.run());

        match res {
            // The connections ended normally (for example, they sent a QUIT message to the server).
            Ok(_) => break,
            // Something went wrong! We'll print the error, and restart the connections.
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn process_msg(client: &IrcClient, message: Message) -> error::Result<()> {
    print!("{}", message);
    if let Command::PRIVMSG(ref target, ref msg) = message.command {
        if msg.contains("pickles") {
            client.send_privmsg(target, "Hi!")?;
        } else if msg.contains("quit") {
            client.send_quit("bye")?;
        }
    }
    Ok(())
}
