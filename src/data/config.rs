use std::collections::HashMap;
use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use serialize::json::decode;

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
            desc: "Failed to decode configuration file.",
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
