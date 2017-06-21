extern crate futures;
extern crate irc;

use std::default::Default;
use std::thread;
use futures::{Future, Stream};
use irc::client::async::IrcServer;
use irc::client::data::Config;
use irc::proto::{CapSubCommand, Command};

fn main() {
    let config = Config {
        nickname: Some("pickles".to_owned()),
        alt_nicks: Some(vec!["bananas".to_owned(), "apples".to_owned()]),
        server: Some("chat.freenode.net".to_owned()),
        channels: Some(vec!["##yulli".to_owned()]),
        ..Default::default()
    };

    let mut server = IrcServer::new(config).unwrap();
    thread::sleep_ms(100);
    server.send(Command::CAP(None, CapSubCommand::END, None, None)).unwrap();
    server.send(Command::NICK("aatxebot".to_owned())).unwrap();
    server.send(Command::USER("aatxebot".to_owned(), "0".to_owned(), "aatxebot".to_owned())).unwrap();
    thread::sleep_ms(100);
    server.send(Command::JOIN("##yulli".to_owned(), None, None)).unwrap();
    server.recv().for_each(|msg| {
        print!("{}", msg);
        match msg.command {
            Command::PRIVMSG(ref target, ref msg) => {
                if msg.contains("pickles") {
                    server.send(Command::PRIVMSG(target.to_owned(), "Hi!".to_owned())).unwrap();
                }
            }
            _ => (),
        }
        Ok(())
    }).wait().unwrap();
}
