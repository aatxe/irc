use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use serialize::json::{decode};

pub trait IrcWriter: Writer + Sized + 'static {}
impl<T> IrcWriter for T where T: Writer + Sized + 'static {}
pub trait IrcReader: Reader + Sized + Clone + 'static {}
impl<T> IrcReader for T where T: Reader + Sized + Clone + 'static {}


#[deriving(Show, PartialEq)]
pub struct Message<'a> {
    pub source: Option<&'a str>,
    pub command: &'a str,
    pub args: &'a [&'a str],
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

#[deriving(Clone, Decodable)]
pub struct Config {
    pub owners: Vec<String>,
    pub nickname: String,
    pub username: String,
    pub realname: String,
    pub password: String,
    pub server: String,
    pub port: u16,
    pub channels: Vec<String>,
}

impl Config {
    pub fn load() -> IoResult<Config> {
        let mut file = try!(File::open(&Path::new("config.json")));
        let data = try!(file.read_to_string());
        decode(data.as_slice()).map_err(|e| IoError {
            kind: InvalidInput,
            desc: "Decoder error",
            detail: Some(e.to_string()),
        })
    }

    pub fn is_owner(&self, nickname: &str) -> bool {
        self.owners.as_slice().contains(&String::from_str(nickname))
    }
}

#[cfg(test)]
mod test {
    use super::{Config, Message};

    #[test]
    fn new_message() {
        let args = ["flare.to.ca.fyrechat.net"];
        let m = Message::new(None, "PING", args);
        assert_eq!(m, Message {
            source: None,
            command: "PING",
            args: args,
        });
    }

    #[test]
    fn load_config() {
        assert!(Config::load().is_ok());
    }

    #[test]
    fn is_owner() {
        let cfg = Config::load().unwrap();
        assert!(cfg.is_owner("test"));
        assert!(!cfg.is_owner("test2"));
    }
}
