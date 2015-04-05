//! JSON configuration files using libserialize.
#![stable]
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use rustc_serialize::json::decode;

/// Configuration data.
#[derive(Clone, RustcDecodable, Default, PartialEq, Debug)]
#[stable]
pub struct Config {
    /// A list of the owners of the bot by nickname.
    #[stable]
    pub owners: Option<Vec<String>>,
    /// The bot's nickname.
    #[stable]
    pub nickname: Option<String>,
    /// The bot's NICKSERV password.
    #[stable]
    pub nick_password: Option<String>,
    /// Alternative nicknames for the bots, if the default is taken.
    #[stable]
    pub alt_nicks: Option<Vec<String>>,
    /// The bot's username.
    #[stable]
    pub username: Option<String>,
    /// The bot's real name.
    #[stable]
    pub realname: Option<String>,
    /// The server to connect to.
    #[stable]
    pub server: Option<String>,
    /// The port to connect on.
    #[stable]
    pub port: Option<u16>,
    /// The password to connect to the server.
    #[stable]
    pub password: Option<String>,
    /// Whether or not to use SSL.
    /// Bots will automatically panic if this is enabled without SSL support.
    #[stable]
    pub use_ssl: Option<bool>,
    /// The encoding type used for this connection.
    /// This is typically UTF-8, but could be something else.
    #[stable]
    pub encoding: Option<String>,
    /// A list of channels to join on connection.
    #[stable]
    pub channels: Option<Vec<String>>,
    /// User modes to set on connect. Example: "+RB-x"
    #[unstable]
    pub umodes: Option<String>,
    /// The text that'll be sent in response to CTCP USERINFO requests.
    #[stable]
    pub user_info: Option<String>,
    /// A map of additional options to be stored in config.
    #[stable]
    pub options: Option<HashMap<String, String>>,
}

#[stable]
impl Config {
    /// Loads a JSON configuration from the desired path.
    #[stable]
    pub fn load(path: &Path) -> Result<Config> {
        let mut file = try!(File::open(path));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        decode(&data[..]).map_err(|_| 
            Error::new(ErrorKind::InvalidInput, "Failed to decode configuration file.")
        )
    }

    /// Loads a JSON configuration using the string as a UTF-8 path.
    #[stable]
    pub fn load_utf8(path: &str) -> Result<Config> {
        Config::load(Path::new(path))
    }

    /// Determines whether or not the nickname provided is the owner of the bot.
    #[stable]
    pub fn is_owner(&self, nickname: &str) -> bool {
        self.owners.as_ref().map(|o| o.contains(&nickname.to_string())).unwrap()
    }

    /// Gets the nickname specified in the configuration.
    /// This will panic if not specified.
    #[stable]
    pub fn nickname(&self) -> &str {
        self.nickname.as_ref().map(|s| &s[..]).unwrap()
    }

    /// Gets the bot's nickserv password specified in the configuration.
    /// This defaults to an empty string when not specified.
    #[stable]
    pub fn nick_password(&self) -> &str {
        self.nick_password.as_ref().map(|s| &s[..]).unwrap_or("")
    }

    /// Gets the alternate nicknames specified in the configuration.
    /// This defaults to an empty vector when not specified.
    #[stable]
    pub fn get_alternate_nicknames(&self) -> Vec<&str> {
        self.alt_nicks.as_ref().map(|v| v.iter().map(|s| &s[..]).collect()).unwrap_or(vec![])
    }


    /// Gets the username specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    #[stable]
    pub fn username(&self) -> &str {
        self.username.as_ref().map(|s| &s[..]).unwrap_or(self.nickname())
    }

    /// Gets the real name specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    #[stable]
    pub fn real_name(&self) -> &str {
        self.realname.as_ref().map(|s| &s[..]).unwrap_or(self.nickname())
    }

    /// Gets the address of the server specified in the configuration.
    /// This panics when not specified.
    #[stable]
    pub fn server(&self) -> &str {
        self.server.as_ref().map(|s| &s[..]).unwrap()
    }

    /// Gets the port of the server specified in the configuration.
    /// This defaults to 6667 (or 6697 if use_ssl is specified as true) when not specified.
    #[stable]
    pub fn port(&self) -> u16 {
        self.port.as_ref().map(|p| *p).unwrap_or(if self.use_ssl() {
            6697
        } else {
            6667
        })
    }

    /// Gets the server password specified in the configuration.
    /// This defaults to a blank string when not specified.
    #[stable]
    pub fn password(&self) -> &str {
        self.password.as_ref().map(|s| &s[..]).unwrap_or("")
    }

    /// Gets whether or not to use SSL with this connection.
    /// This defaults to false when not specified.
    #[stable]
    pub fn use_ssl(&self) -> bool {
        self.use_ssl.as_ref().map(|u| *u).unwrap_or(false)
    }

    /// Gets the encoding to use for this connection. This requires the encode feature to work.
    /// This defaults to UTF-8 when not specified.
    #[stable]
    pub fn encoding(&self) -> &str {
        self.encoding.as_ref().map(|s| &s[..]).unwrap_or("UTF-8")
    }

    /// Gets the channels to join upon connection.
    /// This defaults to an empty vector if it's not specified.
    #[stable]
    pub fn channels(&self) -> Vec<&str> {
        self.channels.as_ref().map(|v| v.iter().map(|s| &s[..]).collect()).unwrap_or(vec![])    
    }

    /// Gets the user modes to set on connect specified in the configuration.
    /// This defaults to an empty string when not specified.
    #[unstable = "Feature is still relatively new."]
    pub fn umodes(&self) -> &str {
        self.umodes.as_ref().map(|s| &s[..]).unwrap_or("")
    }

    /// Gets the string to be sent in response to CTCP USERINFO requests.
    /// This defaults to an empty string when not specified.
    #[stable]
    pub fn user_info(&self) -> &str {
        self.user_info.as_ref().map(|s| &s[..]).unwrap_or("")
    }

    /// Looks up the specified string in the options map.
    /// This uses indexing, and thus panics when the string is not present.
    /// This will also panic if used and there are no options.
    #[stable]
    pub fn get_option(&self, option: &str) -> &str {
        self.options.as_ref().map(|o| &o[&option.to_owned()][..]).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::Config;
    use std::collections::HashMap;
    use std::default::Default;
    use std::path::Path;

    #[test]
    fn load() {
        let cfg = Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            nick_password: None,
            alt_nicks: None,
            username: Some(format!("test")),
            realname: Some(format!("test")),
            password: Some(String::new()),
            umodes: Some(format!("+BR")),
            server: Some(format!("irc.test.net")),
            port: Some(6667),
            use_ssl: Some(false),
            encoding: Some(format!("UTF-8")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            user_info: None,
            options: Some(HashMap::new()),
        };
        assert_eq!(Config::load(Path::new("client_config.json")).unwrap(), cfg);
    }

    #[test]
    fn load_utf8() {
        let cfg = Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            nick_password: None,
            alt_nicks: None,
            username: Some(format!("test")),
            realname: Some(format!("test")),
            umodes: Some(format!("+BR")),
            password: Some(String::new()),
            server: Some(format!("irc.test.net")),
            port: Some(6667),
            use_ssl: Some(false),
            encoding: Some(format!("UTF-8")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            user_info: None,
            options: Some(HashMap::new()),
        };
        assert_eq!(Config::load_utf8("client_config.json").unwrap(), cfg);
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
