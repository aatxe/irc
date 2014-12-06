//! JSON configuration files using libserialize.
#![stable]
use std::collections::HashMap;
use std::io::fs::File;
use std::io::{InvalidInput, IoError, IoResult};
use serialize::json::decode;

/// Configuration data.
#[deriving(Clone, Decodable, Default, PartialEq, Show)]
#[unstable]
pub struct Config {
    /// A list of the owners of the bot by nickname.
    pub owners: Option<Vec<String>>,
    /// The bot's nickname.
    pub nickname: Option<String>,
    /// Alternative nicknames for the bots, if the default is taken.
    pub alt_nicks: Option<Vec<String>>,
    /// The bot's username.
    pub username: Option<String>,
    /// The bot's real name.
    pub realname: Option<String>,
    /// The bot's password.
    pub password: Option<String>,
    /// The server to connect to.
    pub server: Option<String>,
    /// The port to connect on.
    pub port: Option<u16>,
    /// Whether or not to use SSL.
    /// Bots will automatically panic if this is enabled without SSL support.
    pub use_ssl: Option<bool>,
    /// The encoding type used for this connection.
    /// This is typically UTF-8, but could be something else.
    pub encoding: Option<String>,
    /// A list of channels to join on connection.
    pub channels: Option<Vec<String>>,
    /// A map of additional options to be stored in config.
    pub options: Option<HashMap<String, String>>,
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
        self.owners.as_ref().map(|o| o.contains(&String::from_str(nickname))).unwrap()
    }

    /// Gets the nickname specified in the configuration.
    /// This will panic if not specified.
    #[experimental]
    pub fn nickname(&self) -> &str {
        self.nickname.as_ref().map(|s| s[]).unwrap()
    }

    /// Gets the alternate nicknames specified in the configuration.
    /// This defaults to an empty vector when not specified.
    #[experimental]
    pub fn get_alternate_nicknames(&self) -> Vec<&str> {
        self.alt_nicks.as_ref().map(|v| v.iter().map(|s| s[]).collect()).unwrap_or(vec![])
    }


    /// Gets the username specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    #[experimental]
    pub fn username(&self) -> &str {
        self.username.as_ref().map(|s| s[]).unwrap_or(self.nickname())
    }

    /// Gets the real name specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    #[experimental]
    pub fn real_name(&self) -> &str {
        self.realname.as_ref().map(|s| s[]).unwrap_or(self.nickname())
    }

    /// Gets the password specified in the configuration.
    /// This defaults to a blank string when not specified.
    #[experimental]
    pub fn password(&self) -> &str {
        self.password.as_ref().map(|s| s[]).unwrap_or("")
    }

    /// Gets the address of the server specified in the configuration.
    /// This panics when not specified.
    #[experimental]
    pub fn server(&self) -> &str {
        self.server.as_ref().map(|s| s[]).unwrap()
    }

    /// Gets the port of the server specified in the configuration.
    /// This defaults to 6667 (or 6697 if use_ssl is specified as true) when not specified.
    #[experimental]
    pub fn port(&self) -> u16 {
        self.port.as_ref().map(|p| *p).unwrap_or(if self.use_ssl() {
            6697
        } else {
            6667
        })
    }

    /// Gets whether or not to use SSL with this connection.
    /// This defaults to false when not specified.
    #[experimental]
    pub fn use_ssl(&self) -> bool {
        self.use_ssl.as_ref().map(|u| *u).unwrap_or(false)
    }

    /// Gets the encoding to use for this connection. This requires the encode feature to work.
    /// This defaults to UTF-8 when not specified.
    #[experimental]
    pub fn encoding(&self) -> &str {
        self.encoding.as_ref().map(|s| s[]).unwrap_or("UTF-8")
    }

    /// Gets the channels to join upon connection.
    /// This defaults to an empty vector if it's not specified.
    #[experimental]
    pub fn channels(&self) -> Vec<&str> {
        self.channels.as_ref().map(|v| v.iter().map(|s| s[]).collect()).unwrap_or(vec![])    
    }

    /// Looks up the specified string in the options map.
    /// This uses indexing, and thus panics when the string is not present.
    /// This will also panic if used and there are no options.
    #[experimental]
    pub fn get_option(&self, option: &str) -> &str {
        self.options.as_ref().map(|o| o[option.into_string()][]).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::Config;
    use std::collections::HashMap;
    use std::default::Default;

    #[test]
    fn load() {
        let cfg = Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            alt_nicks: None,
            username: Some(format!("test")),
            realname: Some(format!("test")),
            password: Some(String::new()),
            server: Some(format!("irc.test.net")),
            port: Some(6667),
            use_ssl: Some(false),
            encoding: Some(format!("UTF-8")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            options: Some(HashMap::new()),
        };
        assert_eq!(Config::load(Path::new("config.json")), Ok(cfg));
    }

    #[test]
    fn load_utf8() {
        let cfg = Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            alt_nicks: None,
            username: Some(format!("test")),
            realname: Some(format!("test")),
            password: Some(String::new()),
            server: Some(format!("irc.test.net")),
            port: Some(6667),
            use_ssl: Some(false),
            encoding: Some(format!("UTF-8")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            options: Some(HashMap::new()),
        };
        assert_eq!(Config::load_utf8("config.json"), Ok(cfg));
    }


    #[test]
    fn is_owner() {
        let cfg = Config {
            owners: Some(vec![format!("test"), format!("test2")]),
            .. Default::default()
        };
        assert!(cfg.is_owner("test"));
        assert!(cfg.is_owner("test2"));
        assert!(!cfg.is_owner("test3"));
    }

    #[test]
    fn get_option() {
        let cfg = Config {
            options: {
                let mut map = HashMap::new();
                map.insert(format!("testing"), format!("test"));
                Some(map)
            },
            .. Default::default()
        };
        assert_eq!(cfg.get_option("testing"), "test");
    }
}
