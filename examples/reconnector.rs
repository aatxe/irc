extern crate irc;

use std::default::Default;
use irc::error;
#[cfg(feature = "client")]
use irc::client::prelude::*;

#[cfg(feature = "client")]
fn main() {
    let cfg1 = Config {
        nickname: Some("pickles".to_owned()),
        server: Some("irc.mozilla.org".to_owned()),
        channels: Some(vec!["#rust-spam".to_owned()]),
        ..Default::default()
    };

    let cfg2 = Config {
        nickname: Some("bananas".to_owned()),
        server: Some("irc.mozilla.org".to_owned()),
        channels: Some(vec!["#rust-spam".to_owned()]),
        ..Default::default()
    };

    let configs = vec![cfg1, cfg2];

    let mut reactor = IrcReactor::new().unwrap();

    loop {
        let res = configs.iter().fold(Ok(()), |acc, config| {
            acc.and(
                reactor.prepare_client_and_connect(config).and_then(|client| {
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

#[cfg(feature = "client")]
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

#[cfg(not(feature = "client"))]
fn main() {
    eprintln!("built without client support")
}
