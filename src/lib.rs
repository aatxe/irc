#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate serialize;

use std::cell::RefCell;
use std::io::{BufferedReader, InvalidInput, IoError, IoResult};
use data::{Message, Config};
use conn::{Connection, connect, send};

pub mod conn;
pub mod data;

pub struct Bot<'a> {
    pub conn: Connection,
    pub config: Config,
    process: RefCell<|&Bot, &str, &str, &[&str]|:'a -> IoResult<()>>,
}

impl<'a> Bot<'a> {
    pub fn new(process: |&Bot, &str, &str, &[&str]|:'a -> IoResult<()>) -> IoResult<Bot<'a>> {
        let config = try!(Config::load());
        let conn = try!(connect(config.server.as_slice(), config.port));
        Ok(Bot {
            conn: conn,
            config: config,
            process: RefCell::new(process),
        })
    }

    pub fn send_nick(&self, nick: &str) -> IoResult<()> {
        send(&self.conn, Message::new(None, "NICK", [nick]))
    }

    pub fn send_user(&self, username: &str, real_name: &str) -> IoResult<()> {
        send(&self.conn, Message::new(None, "USER", [username, "0", "*", real_name]))
    }

    pub fn send_join(&self, chan: &str) -> IoResult<()> {
        send(&self.conn, Message::new(None, "JOIN", [chan.as_slice()]))
    }

    pub fn identify(&self) -> IoResult<()> {
        self.send_nick(self.config.nickname.as_slice());
        self.send_user(self.config.username.as_slice(), self.config.realname.as_slice())
    }

    pub fn output(&mut self) {
        let mut reader = { let Connection(ref tcp) = self.conn; BufferedReader::new(tcp.clone()) };
        for line in reader.lines() {
            match line {
                Ok(ln) => {
                    let (source, command, args) = process(ln.as_slice()).unwrap();
                    self.handle_command(source, command, args.as_slice());
                    println!("{}", ln)
                },
                Err(e) => println!("Shit, you're fucked! {}", e),
            }
        }
    }

    fn handle_command(&mut self, source: &str, command: &str, args: &[&str]) -> IoResult<()> {
        match (command, args) {
            ("PING", [msg]) => {
                try!(send(&self.conn, Message::new(None, "PONG", [msg])));
            },
            ("376", _) => { // End of MOTD
                for chan in self.config.channels.iter() {
                    try!(self.send_join(chan.as_slice()));
                }
            },
            ("422", _) => { // Missing MOTD
                for chan in self.config.channels.iter() {
                    try!(self.send_join(chan.as_slice()));
                }
            },
            _ => {
                (*self.process.borrow_mut().deref_mut())(self, source, command, args);
            },
        };
        Ok(())
    }
}

fn process(msg: &str) -> IoResult<(&str, &str, Vec<&str>)> {
    let reg = regex!(r"^(?::([^ ]+) )?([^ ]+)(.*)");
    let cap = match reg.captures(msg) {
        Some(x) => x,
        None => return Err(IoError {
            kind: InvalidInput,
            desc: "Failed to parse line",
            detail: None,
        }),
    };
    let source = cap.at(1);
    let command = cap.at(2);
    let args = parse_args(cap.at(3));
    Ok((source, command, args))
}

fn parse_args(line: &str) -> Vec<&str> {
    let reg = regex!(r" ([^: ]+)| :(.*)$");
    reg.captures_iter(line).map(|cap| {
        match cap.at(1) {
            "" => cap.at(2),
            x => x,
        }
    }).collect()
}
