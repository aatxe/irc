use std::collections::HashMap;
use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use serialize::json::{decode};

pub trait IrcWriter: Writer + Sized + 'static {}
impl<T> IrcWriter for T where T: Writer + Sized + 'static {}
pub trait IrcReader: Buffer + Sized + 'static {}
impl<T> IrcReader for T where T: Buffer + Sized + 'static {}

#[deriving(PartialEq, Clone, Show)]
pub struct User {
    name: String,
    access_level: AccessLevel,
}

impl User {
    pub fn new(name: &str) -> User {
        let rank = AccessLevel::from_str(name);
        User {
            name: if let Member = rank {
                name.into_string()
            } else {
                name[1..].into_string()
            },
            access_level: rank,
        }
    }
}

#[deriving(PartialEq, Clone, Show)]
pub enum AccessLevel {
    Owner,
    Admin,
    Oper,
    HalfOp,
    Voice,
    Member,
}

impl AccessLevel {
    pub fn from_str(s: &str) -> AccessLevel {
        if s.len() == 0 { Member } else {
            match s.char_at(0) {
                '~' => Owner,
                '&' => Admin,
                '@' => Oper,
                '%' => HalfOp,
                '+' => Voice,
                 _  => Member,
            }
        }
    }
}

#[deriving(Show, PartialEq)]
pub struct Message {
    pub source: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    pub colon_flag: Option<bool>,
}

impl<'a> Message {
    pub fn new(source: Option<&'a str>, command: &'a str, args: Vec<&'a str>, colon_flag: Option<bool>) -> Message {
        Message {
            source: source.map(|s: &str| s.into_string()),
            command: command.into_string(),
            args: args.into_iter().map(|s: &str| s.into_string()).collect(),
            colon_flag: colon_flag,
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
    pub options: HashMap<String, String>,
}

impl Config {
    pub fn load(path: Path) -> IoResult<Config> {
        let mut file = try!(File::open(&path));
        let data = try!(file.read_to_string());
        decode(data[]).map_err(|e| IoError {
            kind: InvalidInput,
            desc: "Decoder error",
            detail: Some(e.to_string()),
        })
    }

    pub fn load_utf8(path: &str) -> IoResult<Config> {
        Config::load(Path::new(path))
    }

    pub fn is_owner(&self, nickname: &str) -> bool {
        self.owners[].contains(&String::from_str(nickname))
    }
}
