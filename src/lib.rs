#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate serialize;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufferedReader, BufferedWriter, InvalidInput, IoError, IoResult, TcpStream};
use std::vec::Vec;
use conn::{Conn, Connection};
use data::{Config, IrcReader, IrcWriter, Message};

pub mod conn;
pub mod data;

pub trait Bot<'a> {
    fn send_nick(&self, nick: &str) -> IoResult<()>;
    fn send_user(&self, username: &str, real_name: &str) -> IoResult<()>;
    fn send_join(&self, chan: &str) -> IoResult<()>;
    fn send_mode(&self, chan: &str, mode: &str) -> IoResult<()>;
    fn send_topic(&self, chan: &str, topic: &str) -> IoResult<()>;
    fn send_invite(&self, person: &str, chan: &str) -> IoResult<()>;
    fn send_privmsg(&self, chan: &str, msg: &str) -> IoResult<()>;
    fn identify(&self) -> IoResult<()>;
    fn output(&mut self) -> IoResult<()>;
    fn config(&self) -> &Config;
}

pub struct IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub conn: RefCell<Connection<T, U>>,
    pub config: Config,
    process: RefCell<|&IrcBot<T, U>, &str, &str, &[&str]|:'a -> IoResult<()>>,
    pub chanlists: HashMap<String, Vec<String>>,
}

impl<'a> IrcBot<'a, BufferedWriter<TcpStream>, TcpStream> {
    pub fn new(process: |&IrcBot<BufferedWriter<TcpStream>, TcpStream>, &str, &str, &[&str]|:'a -> IoResult<()>) -> IoResult<IrcBot<'a, BufferedWriter<TcpStream>, TcpStream>> {
        let config = try!(Config::load());
        let conn = try!(Connection::connect(config.server.as_slice(), config.port));
        Ok(IrcBot {
            conn: RefCell::new(conn),
            config: config,
            process: RefCell::new(process),
            chanlists: HashMap::new(),
        })
    }
}

impl<'a, T, U> Bot<'a> for IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn send_nick(&self, nick: &str) -> IoResult<()> {
        Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "NICK", [nick]))
    }

    fn send_user(&self, username: &str, real_name: &str) -> IoResult<()> {
        Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "USER", [username, "0", "*", real_name]))
    }

    fn send_join(&self, chan: &str) -> IoResult<()> {
        Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "JOIN", [chan.as_slice()]))
    }

    fn send_mode(&self, chan: &str, mode: &str) -> IoResult<()> {
        Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "MODE", [chan.as_slice(), mode.as_slice()]))
    }

    fn send_topic(&self, chan: &str, topic: &str) -> IoResult<()> {
        Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "TOPIC", [chan.as_slice(), topic.as_slice()]))
    }

    fn send_invite(&self, person: &str, chan: &str) -> IoResult<()> {
        Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "INVITE", [person.as_slice(), chan.as_slice()]))
    }

    fn send_privmsg(&self, chan: &str, msg: &str) -> IoResult<()> {
        Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "PRIVMSG", [chan.as_slice(), msg.as_slice()]))
    }

    fn identify(&self) -> IoResult<()> {
        try!(self.send_nick(self.config.nickname.as_slice()));
        self.send_user(self.config.username.as_slice(), self.config.realname.as_slice())
    }

    fn output(&mut self) -> IoResult<()> {
        let mut reader = match self.conn.borrow_mut().deref_mut() {
            &Conn(_, ref recv) => BufferedReader::new(recv.clone()),
        };
        for line in reader.lines() {
            match line {
                Ok(ln) => {
                    let (source, command, args) = try!(process(ln.as_slice()));
                    try!(self.handle_command(source, command, args.as_slice()));
                    println!("{}", ln)
                },
                Err(e) => println!("Shit, you're fucked! {}", e),
            }
        }
        Ok(())
    }

    fn config(&self) -> &Config {
        &self.config
    }
}

impl<'a, T, U> IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn handle_command(&mut self, source: &str, command: &str, args: &[&str]) -> IoResult<()> {
        match (command, args) {
            ("PING", [msg]) => {
                try!(Connection::send(self.conn.borrow_mut().deref_mut(), Message::new(None, "PONG", [msg])));
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
            ("353", [_, _, chan, users]) => { // /NAMES
                for user in users.split_str(" ") {
                    if !match self.chanlists.find_mut(&String::from_str(chan)) {
                        Some(vec) => {
                            vec.push(String::from_str(user));
                            true
                        },
                        None => false,
                    } {
                        self.chanlists.insert(String::from_str(chan), vec!(String::from_str(user)));
                    }
                }
            },
            ("JOIN", [chan]) => {
                match self.chanlists.find_mut(&String::from_str(chan)) {
                    Some(vec) => {
                        match source.find('!') {
                            Some(i) => vec.push(String::from_str(source.slice_to(i))),
                            None => (),
                        };
                    },
                    None => (),
                }
            },
            ("PART", [chan, _]) => {
                match self.chanlists.find_mut(&String::from_str(chan)) {
                    Some(vec) => {
                        match source.find('!') {
                            Some(i) => {
                                match vec.as_slice().position_elem(&String::from_str(source.slice_to(i))) {
                                    Some(n) => {
                                        vec.swap_remove(n);
                                    },
                                    None => (),
                                };
                            },
                            None => (),
                        };
                    },
                    None => (),
                }
            },
            _ => {
                try!((*self.process.borrow_mut().deref_mut())(self, source, command, args));
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
    let reg = regex!(r" ([^: ]+)| :([^\r\n]*)[\r\n]*$");
    reg.captures_iter(line).map(|cap| {
        match cap.at(1) {
            "" => cap.at(2),
            x => x,
        }
    }).collect()
}

#[test]
fn process_line_test() {
    let res = process(":flare.to.ca.fyrechat.net 353 pickles = #pickles :pickles awe\r\n").unwrap();
    let (source, command, args) = res;
    assert_eq!(source, "flare.to.ca.fyrechat.net");
    assert_eq!(command, "353");
    assert_eq!(args, vec!["pickles", "=", "#pickles", "pickles awe"]);

    let res = process("PING :flare.to.ca.fyrechat.net\r\n").unwrap();
    let (source, command, args) = res;
    assert_eq!(source, "");
    assert_eq!(command, "PING");
    assert_eq!(args, vec!["flare.to.ca.fyrechat.net"]);
}

#[test]
fn process_args_test() {
    let res = parse_args("PRIVMSG #vana :hi");
    assert_eq!(res, vec!["#vana", "hi"])
}
