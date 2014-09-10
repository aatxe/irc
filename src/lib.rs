#![feature(phase)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;

use std::io::{BufferedReader, BufferedWriter, InvalidInput, IoError, IoResult, TcpStream};

pub struct Bot {
    pub sock: TcpStream
}

impl Bot {
    pub fn new() -> Bot {
        let sock = TcpStream::connect("irc.fyrechat.net", 6667).unwrap();
        Bot {
            sock: sock,
        }
    }

    pub fn identify(&mut self) {
        let mut writer = BufferedWriter::new(self.sock.clone());
        writer.write_str("NICK :pickles\r\n").unwrap();
        writer.write_str("USER pickles 0 * :pickles\r\n").unwrap();
        writer.flush().unwrap();
    }

    pub fn output(&mut self) {
        let mut reader = BufferedReader::new(self.sock.clone());
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

    fn handle_command(&mut self, source: &str, command: &str, args: &[&str]) -> () {
        match (command, args) {
            ("PING", [msg]) => {
                self.send("PONG", msg);
            },
            ("376", _) => {
                self.send("JOIN", "#vana");
            },
            ("PRIVMSG", [channel, msg]) => {
                if msg.contains("pickles") && msg.contains("hi") {
                    self.send("PRIVMSG #vana", "hi");
                }

                if msg.starts_with(". ") {
                    self.send("PRIVMSG #vana", msg.slice_from(2));
                }
            },
            _ => (),
        }
    }

    fn send(&mut self, command: &str, arg: &str) {
        let mut writer = BufferedWriter::new(self.sock.clone());
        write!(writer, "{} :{}\r\n", command, arg);
        writer.flush().unwrap();
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
