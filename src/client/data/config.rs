//! JSON configuration files using serde
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use serde_json;

/// Configuration data.
#[derive(Clone, Deserialize, Serialize, Default, PartialEq, Debug)]
pub struct Config {
    /// A list of the owners of the client by nickname (for bots).
    pub owners: Option<Vec<String>>,
    /// The client's nickname.
    pub nickname: Option<String>,
    /// The client's NICKSERV password.
    pub nick_password: Option<String>,
    /// Alternative nicknames for the client, if the default is taken.
    pub alt_nicks: Option<Vec<String>>,
    /// The client's username.
    pub username: Option<String>,
    /// The client's real name.
    pub realname: Option<String>,
    /// The server to connect to.
    pub server: Option<String>,
    /// The port to connect on.
    pub port: Option<u16>,
    /// The password to connect to the server.
    pub password: Option<String>,
    /// Whether or not to use SSL.
    /// Clients will automatically panic if this is enabled without SSL support.
    pub use_ssl: Option<bool>,
    /// The encoding type used for this connection.
    /// This is typically UTF-8, but could be something else.
    pub encoding: Option<String>,
    /// A list of channels to join on connection.
    pub channels: Option<Vec<String>>,
    /// A mapping of channel names to keys for join-on-connect.
    pub channel_keys: Option<HashMap<String, String>>,
    /// User modes to set on connect. Example: "+RB-x"
    pub umodes: Option<String>,
    /// The text that'll be sent in response to CTCP USERINFO requests.
    pub user_info: Option<String>,
    /// The text that'll be sent in response to CTCP VERSION requests.
    pub version: Option<String>,
    /// The text that'll be sent in response to CTCP SOURCE requests.
    pub source: Option<String>,
    /// The amount of inactivity in seconds before the client will ping the server.
    pub ping_time: Option<u32>,
    /// The amount of time in seconds for a client to reconnect due to no ping response.
    pub ping_timeout: Option<u32>,
    /// Whether the client should use NickServ GHOST to reclaim its primary nickname if it is in use.
    /// This has no effect if `nick_password` is not set.
    pub should_ghost: Option<bool>,
    /// The command(s) that should be sent to NickServ to recover a nickname. The nickname and password will be appended in that order after the command.
    /// E.g. `["RECOVER", "RELEASE"]` means `RECOVER nick pass` and `RELEASE nick pass` will be sent in that order.
    pub ghost_sequence: Option<Vec<String>>,
    /// A map of additional options to be stored in config.
    pub options: Option<HashMap<String, String>>,
}

impl Config {
    /// Loads a JSON configuration from the desired path.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = try!(File::open(path));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        serde_json::from_str(&data[..]).map_err(|_|
            Error::new(ErrorKind::InvalidInput, "Failed to decode configuration file.")
        )
    }

    /// Saves a JSON configuration to the desired path.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = try!(File::create(path));
        file.write_all(try!(serde_json::to_string(self).map_err(|_|
            Error::new(ErrorKind::InvalidInput, "Failed to encode configuration file.")
        )).as_bytes())
    }

    /// Determines whether or not the nickname provided is the owner of the bot.
    pub fn is_owner(&self, nickname: &str) -> bool {
        self.owners.as_ref().map(|o| o.contains(&nickname.to_owned())).unwrap()
    }

    /// Gets the nickname specified in the configuration.
    /// This will panic if not specified.
    pub fn nickname(&self) -> &str {
        self.nickname.as_ref().map(|s| &s[..]).unwrap()
    }

    /// Gets the bot's nickserv password specified in the configuration.
    /// This defaults to an empty string when not specified.
    pub fn nick_password(&self) -> &str {
        self.nick_password.as_ref().map_or("", |s| &s[..])
    }

    /// Gets the alternate nicknames specified in the configuration.
    /// This defaults to an empty vector when not specified.
    pub fn alternate_nicknames(&self) -> Vec<&str> {
        self.alt_nicks.as_ref().map_or(vec![], |v| v.iter().map(|s| &s[..]).collect())
    }


    /// Gets the username specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    pub fn username(&self) -> &str {
        self.username.as_ref().map_or(self.nickname(), |s| &s[..])
    }

    /// Gets the real name specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    pub fn real_name(&self) -> &str {
        self.realname.as_ref().map_or(self.nickname(), |s| &s[..])
    }

    /// Gets the address of the server specified in the configuration.
    /// This panics when not specified.
    pub fn server(&self) -> &str {
        self.server.as_ref().map(|s| &s[..]).unwrap()
    }

    /// Gets the port of the server specified in the configuration.
    /// This defaults to 6667 (or 6697 if use_ssl is specified as true) when not specified.
    pub fn port(&self) -> u16 {
        self.port.as_ref().cloned().unwrap_or(if self.use_ssl() {
            6697
        } else {
            6667
        })
    }

    /// Gets the server password specified in the configuration.
    /// This defaults to a blank string when not specified.
    pub fn password(&self) -> &str {
        self.password.as_ref().map_or("", |s| &s[..])
    }

    /// Gets whether or not to use SSL with this connection.
    /// This defaults to false when not specified.
    pub fn use_ssl(&self) -> bool {
        self.use_ssl.as_ref().cloned().unwrap_or(false)
    }

    /// Gets the encoding to use for this connection. This requires the encode feature to work.
    /// This defaults to UTF-8 when not specified.
    pub fn encoding(&self) -> &str {
        self.encoding.as_ref().map_or("UTF-8", |s| &s[..])
    }

    /// Gets the channels to join upon connection.
    /// This defaults to an empty vector if it's not specified.
    pub fn channels(&self) -> Vec<&str> {
        self.channels.as_ref().map_or(vec![], |v| v.iter().map(|s| &s[..]).collect())
    }


    /// Gets the key for the specified channel if it exists in the configuration.
    pub fn channel_key(&self, chan: &str) -> Option<&str> {
        self.channel_keys.as_ref().and_then(|m| m.get(&chan.to_owned()).map(|s| &s[..]))
    }

    /// Gets the user modes to set on connect specified in the configuration.
    /// This defaults to an empty string when not specified.
    pub fn umodes(&self) -> &str {
        self.umodes.as_ref().map_or("", |s| &s[..])
    }

    /// Gets the string to be sent in response to CTCP USERINFO requests.
    /// This defaults to an empty string when not specified.
    pub fn user_info(&self) -> &str {
        self.user_info.as_ref().map_or("", |s| &s[..])
    }

    /// Gets the string to be sent in response to CTCP VERSION requests.
    /// This defaults to `irc:git:Rust` when not specified.
    pub fn version(&self) -> &str {
        self.version.as_ref().map_or("irc:git:Rust", |s| &s[..])
    }

    /// Gets the string to be sent in response to CTCP SOURCE requests.
    /// This defaults to `https://github.com/aatxe/irc` when not specified.
    pub fn source(&self) -> &str {
        self.source.as_ref().map_or("https://github.com/aatxe/irc", |s| &s[..])
    }

    /// Gets the amount of time in seconds since last activity necessary for the client to ping the
    /// server.
    /// This defaults to 180 seconds when not specified.
    pub fn ping_time(&self) -> u32 {
        self.ping_time.as_ref().cloned().unwrap_or(180)
    }

    /// Gets the amount of time in seconds for the client to reconnect after no ping response.
    /// This defaults to 10 seconds when not specified.
    pub fn ping_timeout(&self) -> u32 {
        self.ping_timeout.as_ref().cloned().unwrap_or(10)
    }

    /// Gets whether or not to attempt nickname reclamation using NickServ GHOST.
    /// This defaults to false when not specified.
    pub fn should_ghost(&self) -> bool {
        self.should_ghost.as_ref().cloned().unwrap_or(false)
    }

    /// Gets the NickServ command sequence to recover a nickname.
    /// This defaults to `["GHOST"]` when not specified.
    pub fn ghost_sequence(&self) -> Vec<&str> {
        self.ghost_sequence.as_ref().map_or(vec!["GHOST"], |v| v.iter().map(|s| &s[..]).collect())
    }

    /// Looks up the specified string in the options map.
    /// This uses indexing, and thus panics when the string is not present.
    /// This will also panic if used and there are no options.
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
            channel_keys: None,
            user_info: None,
            version: None,
            source: None,
            ping_time: None,
            ping_timeout: None,
            should_ghost: None,
            ghost_sequence: None,
            options: Some(HashMap::new()),
        };
        assert_eq!(Config::load(Path::new("client_config.json")).unwrap(), cfg);
    }

    #[test]
    fn load_from_str() {
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
            channel_keys: None,
            user_info: None,
            version: None,
            source: None,
            ping_time: None,
            ping_timeout: None,
            should_ghost: None,
            ghost_sequence: None,
            options: Some(HashMap::new()),
        };
        assert_eq!(Config::load("client_config.json").unwrap(), cfg);
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
