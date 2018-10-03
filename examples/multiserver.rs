extern crate irc;

use std::default::Default;
use irc::error;
use irc::client::prelude::*;

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

    for config in configs {
        // Immediate errors like failure to resolve the server's domain or to establish any connection will
        // manifest here in the result of prepare_client_and_connect.
        let client = reactor.prepare_client_and_connect(config).unwrap();
        client.identify().unwrap();
        // Here, we tell the reactor to setup this client for future handling (in run) using the specified
        // handler function process_msg.
        reactor.register_client_with_handler(client, process_msg);
    }

    // Runtime errors like a dropped connection will manifest here in the result of run.
    reactor.run().unwrap();
}

fn process_msg(client: &IrcClient, message: Message) -> error::Result<()> {
    print!("{}", message);
    match message.command {
        Command::PRIVMSG(ref target, ref msg) => {
            if msg.contains("pickles") {
                client.send_privmsg(target, "Hi!")?;
            }
        }
        _ => (),
    }
    Ok(())
}
