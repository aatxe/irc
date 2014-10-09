use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use {Bot, process};
use conn::Connection;
use data::{Config, IrcReader, IrcWriter, Message};

pub struct IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub conn: Connection<T, U>,
    pub config: Config,
    process: RefCell<|&IrcBot<T, U>, &str, &str, &[&str]|:'a -> IoResult<()>>,
    pub chanlists: RefCell<HashMap<String, Vec<String>>>,
}

impl<'a> IrcBot<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    pub fn new(process: |&IrcBot<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>, &str, &str, &[&str]|:'a -> IoResult<()>) -> IoResult<IrcBot<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let config = try!(Config::load());
        let conn = try!(Connection::connect(config.server.as_slice(), config.port));
        Ok(IrcBot {
            conn: conn,
            config: config,
            process: RefCell::new(process),
            chanlists: RefCell::new(HashMap::new()),
        })
    }
}

impl<'a, T, U> Bot<'a> for IrcBot<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn send_nick(&self, nick: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "NICK", [nick]))
    }

    fn send_user(&self, username: &str, real_name: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "USER", [username, "0", "*", real_name]))
    }

    fn send_join(&self, chan: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "JOIN", [chan.as_slice()]))
    }

    fn send_mode(&self, chan: &str, mode: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "MODE", [chan.as_slice(), mode.as_slice()]))
    }

    fn send_topic(&self, chan: &str, topic: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "TOPIC", [chan.as_slice(), topic.as_slice()]))
    }

    fn send_invite(&self, person: &str, chan: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "INVITE", [person.as_slice(), chan.as_slice()]))
    }

    fn send_privmsg(&self, chan: &str, msg: &str) -> IoResult<()> {
        self.conn.send(Message::new(None, "PRIVMSG", [chan.as_slice(), msg.as_slice()]))
    }

    fn identify(&self) -> IoResult<()> {
        try!(self.send_nick(self.config.nickname.as_slice()));
        self.send_user(self.config.username.as_slice(), self.config.realname.as_slice())
    }

    fn output(&mut self) -> IoResult<()> {
        let mut reader = self.conn.reader();
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
    pub fn from_connection(conn: Connection<T, U>, process: |&IrcBot<T, U>, &str, &str, &[&str]|:'a -> IoResult<()>) -> IoResult<IrcBot<'a, T, U>> {
        let config = try!(Config::load());
        Ok(IrcBot {
            conn: conn,
            config: config,
            process: RefCell::new(process),
            chanlists: RefCell::new(HashMap::new()),
        })
    }

    fn handle_command(&self, source: &str, command: &str, args: &[&str]) -> IoResult<()> {
        match (command, args) {
            ("PING", [msg]) => {
                try!(self.conn.send(Message::new(None, "PONG", [msg])));
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
                    if !match self.chanlists.borrow_mut().find_mut(&String::from_str(chan)) {
                        Some(vec) => {
                            vec.push(String::from_str(user));
                            true
                        },
                        None => false,
                    } {
                        self.chanlists.borrow_mut().insert(String::from_str(chan), vec!(String::from_str(user)));
                    }
                }
            },
            ("JOIN", [chan]) => {
                match self.chanlists.borrow_mut().find_mut(&String::from_str(chan)) {
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
                match self.chanlists.borrow_mut().find_mut(&String::from_str(chan)) {
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
