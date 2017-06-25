extern crate futures;
extern crate irc;

use std::default::Default;
use futures::stream::MergedItem;
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
        nickname: Some("pickles".to_owned()),
        server: Some("irc.pdgn.co".to_owned()),
        channels: Some(vec!["#irc-crate".to_owned()]),
        use_ssl: Some(true),
        ..Default::default()
    };

    let server1 = IrcServer::from_config(cfg1).unwrap();
    let server2 = IrcServer::from_config(cfg2).unwrap();
    server1.identify().unwrap();
    server2.identify().unwrap();

    server1.stream().merge(server2.stream()).for_each(|pair| match pair {
        MergedItem::First(message) => process_msg(&server1, message),
        MergedItem::Second(message) => process_msg(&server2, message),
        MergedItem::Both(msg1, msg2) => {
            process_msg(&server1, msg1).unwrap();
            process_msg(&server2, msg2)
        }
    }).wait().unwrap()
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
