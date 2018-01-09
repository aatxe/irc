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
                reactor.prepare_server_and_connect(config).and_then(|server| {
                    server.identify().and(Ok(server))
                }).and_then(|server| {
                    reactor.register_server_with_handler(server, process_msg);
                    Ok(())
                })
            )
        }).and_then(|()| reactor.run());

        match res {
            Ok(_) => break,
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn process_msg(server: &IrcServer, message: Message) -> error::Result<()> {
    print!("{}", message);
    match message.command {
        Command::PRIVMSG(ref target, ref msg) => {
            if msg.contains("pickles") {
                server.send_privmsg(target, "Hi!")?;
            } else if msg.contains("quit") {
                server.send_quit("bye")?;
            }
        }
        _ => (),
    }
    Ok(())
}
