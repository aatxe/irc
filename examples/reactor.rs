extern crate irc;

// This example is meant to be a direct analogue to simple.rs using the reactor API.
#[cfg(feature = "client")]
fn main() {
    use std::default::Default;
    use irc::client::prelude::*;

    let config = Config {
        nickname: Some("pickles".to_owned()),
        alt_nicks: Some(vec!["bananas".to_owned(), "apples".to_owned()]),
        server: Some("irc.mozilla.org".to_owned()),
        channels: Some(vec!["#rust-spam".to_owned()]),
        ..Default::default()
    };

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, |client, message| {
        print!("{}", message);
        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if msg.contains("pickles") {
                client.send_privmsg(target, "Hi!")?;
            }
        }
        Ok(())
    });

    reactor.run().unwrap();
}

#[cfg(not(feature = "client"))]
fn main() {
    eprintln!("built without client support")
}
