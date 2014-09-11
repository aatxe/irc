#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

use std::io::{BufferedReader, BufferedWriter, InvalidInput, IoError, IoResult, TcpStream};

pub struct Connection(TcpStream);

pub fn connect(host: &str, port: u16) -> IoResult<Connection> {
    let socket = try!(TcpStream::connect(host, port));
    Ok(Connection(socket))
}

fn send_internal(conn: &Connection, msg: &str) -> IoResult<()> {
    match conn {
        &Connection(ref tcp) => {
            let mut writer = BufferedWriter::new(tcp.clone());
            writer.write_str(msg);
            writer.flush()
        },
    }
}

pub struct Message<'a> {
    source: Option<&'a str>,
    command: &'a str,
    args: &'a [&'a str],
}

impl<'a> Message<'a> {
    pub fn new(source: Option<&'a str>, command: &'a str, args: &'a [&'a str]) -> Message<'a> {
        Message {
            source: source,
            command: command,
            args: args,
        }
    }
}

pub fn send(conn: &Connection, msg: Message) -> IoResult<()> {
    let arg_string = msg.args.init().connect(" ").append(" :").append(*msg.args.last().unwrap());
    send_internal(conn, msg.command.to_string().append(" ").append(arg_string.as_slice()).as_slice())
}

pub struct Bot {
    pub conn: Connection,
}

impl Bot {
    pub fn new() -> IoResult<Bot> {
        let conn = try!(connect("irc.fyrechat.net", 6667));
        Ok(Bot {
            conn: conn,
        })
    }

    pub fn send_nick(&mut self, nick: &str) -> IoResult<()> {
        send(&self.conn, Message::new(None, "NICK", [nick]))
    }

    pub fn send_user(&mut self, username: &str, real_name: &str) -> IoResult<()> {
        send(&self.conn, Message::new(None, "USER", [username, "0", "*", real_name]))
    }

    pub fn identify(&mut self) -> IoResult<()> {
        self.send_nick("pickles");
        self.send_user("pickles", "pickles")
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
            ("376", _) => {
                try!(send(&self.conn, Message::new(None, "JOIN", ["#vana"])));
            },
            ("PRIVMSG", [_, msg]) => {
                if msg.contains("pickles") && msg.contains("hi") {
                    try!(send(&self.conn, Message::new(None, "PRIVMSG", ["#vana", "hi"])));
                } else if msg.starts_with(". ") {
                    try!(send(&self.conn, Message::new(None, "PRIVMSG", ["#vana", msg.slice_from(2)])));
                };
            },
            _ => (),
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
