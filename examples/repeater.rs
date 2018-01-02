extern crate irc;

use std::default::Default;
use irc::client::prelude::*;

fn main() {
    let config = Config {
        nickname: Some("repeater".to_owned()),
        alt_nicks: Some(vec!["blaster".to_owned(), "smg".to_owned()]),
        server: Some("irc.mozilla.org".to_owned()),
        use_ssl: Some(true),
        channels: Some(vec!["#rust-spam".to_owned()]),
        burst_window_length: Some(4),
        max_messages_in_burst: Some(4),
        ..Default::default()
    };

    let server = IrcServer::from_config(config).unwrap();
    server.identify().unwrap();

    server.for_each_incoming(|message| {
        print!("{}", message);
        match message.command {
            Command::PRIVMSG(ref target, ref msg) => {
                if msg.starts_with(server.current_nickname()) {
                    let tokens: Vec<_> = msg.split(' ').collect();
                    if tokens.len() > 2 {
                        let n = tokens[0].len() + tokens[1].len() + 2;
                        if let Ok(count) = tokens[1].parse::<u8>() {
                            for _ in 0..count {
                                server.send_privmsg(
                                    message.response_target().unwrap_or(target),
                                    &msg[n..]
                                ).unwrap();
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }).unwrap()
}
