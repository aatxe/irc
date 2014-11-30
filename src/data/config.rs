//! JSON configuration files using libserialize.
#![stable]
use std::collections::HashMap;
use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use serialize::json::decode;

/// Configuration data.
#[deriving(Clone, Decodable, PartialEq, Show)]
#[unstable]
pub struct Config {
    /// A list of the owners of the bot by nickname.
    pub owners: Vec<String>,
    /// The bot's nickname.
    pub nickname: String,
    /// The bot's username.
    pub username: String,
    /// The bot's real name.
    pub realname: String,
    /// The bot's password.
    pub password: String,
    /// The server to connect to.
    pub server: String,
    /// The port to connect on.
    pub port: u16,
    /// Whether or not to use SSL.
    /// Bots will automatically panic if this is enabled without SSL support.
    pub use_ssl: bool,
    /// The encoding type used for this connection.
    /// This is typically UTF-8, but could be something else.
    #[cfg(feature = "encoding")]
    pub encoding: String,
    /// A list of channels to join on connection.
    pub channels: Vec<String>,
    /// A map of additional options to be stored in config.
    pub options: HashMap<String, String>,
}

impl Config {
    /// Loads a JSON configuration from the desired path.
    #[stable]
    pub fn load(path: Path) -> IoResult<Config> {
        let mut file = try!(File::open(&path));
        let data = try!(file.read_to_string());
        decode(data[]).map_err(|e| IoError {
            kind: InvalidInput,
            desc: "Failed to decode configuration file.",
            detail: Some(e.to_string()),
        })
    }

    /// Loads a JSON configuration using the string as a UTF-8 path.
    #[stable]
    pub fn load_utf8(path: &str) -> IoResult<Config> {
        Config::load(Path::new(path))
    }

    /// Determines whether or not the nickname provided is the owner of the bot.
    #[stable]
    pub fn is_owner(&self, nickname: &str) -> bool {
        self.owners[].contains(&String::from_str(nickname))
    }

    /// Looks up the specified string in the options map.
    /// This uses indexing, and thus panics when the string is not present.
    #[experimental]
    pub fn get_option(&self, option: &str) -> &str {
        self.options[option.into_string()][]
    }
}

#[cfg(test)]
mod test {
    use super::Config;
    use std::collections::HashMap;

    #[test]
    #[cfg(feature = "encode")]
    fn load() {
        let cfg = Config {
            owners: vec![format!("test")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            encoding: format!("UTF-8"),
            channels: vec![format!("#test"), format!("#test2")],
            options: HashMap::new(),
        };
        assert_eq!(Config::load(Path::new("config.json")), Ok(cfg));
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn load() {
        let cfg = Config {
            owners: vec![format!("test")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            channels: vec![format!("#test"), format!("#test2")],
            options: HashMap::new(),
        };
        assert_eq!(Config::load(Path::new("config.json")), Ok(cfg));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn load_utf8() {
        let cfg = Config {
            owners: vec![format!("test")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            encoding: format!("UTF-8"),
            channels: vec![format!("#test"), format!("#test2")],
            options: HashMap::new(),
        };
        assert_eq!(Config::load_utf8("config.json"), Ok(cfg));
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn load_utf8() {
        let cfg = Config {
            owners: vec![format!("test")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            channels: vec![format!("#test"), format!("#test2")],
            options: HashMap::new(),
        };
        assert_eq!(Config::load_utf8("config.json"), Ok(cfg));
    }


    #[test]
    #[cfg(feature = "encode")]
    fn is_owner() {
        let cfg = Config {
            owners: vec![format!("test"), format!("test2")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            encoding: format!("UTF-8"),
            channels: Vec::new(),
            options: HashMap::new(),
        };
        assert!(cfg.is_owner("test"));
        assert!(cfg.is_owner("test2"));
        assert!(!cfg.is_owner("test3"));
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn is_owner() {
        let cfg = Config {
            owners: vec![format!("test"), format!("test2")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            channels: vec![format!("#test"), format!("#test2")],
            options: HashMap::new(),
        };
        assert!(cfg.is_owner("test"));
        assert!(cfg.is_owner("test2"));
        assert!(!cfg.is_owner("test3"));
    }

    #[test]
    #[cfg(feature = "encode")]
    fn get_option() {
         let cfg = Config {
            owners: vec![format!("test")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            encoding: format!("UTF-8"),
            channels: vec![format!("#test"), format!("#test2")],
            options: {
                let mut map = HashMap::new();
                map.insert(format!("testing"), format!("test"));
                map
            },
        };
        assert_eq!(cfg.get_option("testing"), "test");
    }

    #[test]
    #[cfg(not(feature = "encode"))]
    fn get_option() {
         let cfg = Config {
            owners: vec![format!("test")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            use_ssl: false,
            channels: vec![format!("#test"), format!("#test2")],
            options: {
                let mut map = HashMap::new();
                map.insert(format!("testing"), format!("test"));
                map
            },
        };
        assert_eq!(cfg.get_option("testing"), "test");
    }
}
