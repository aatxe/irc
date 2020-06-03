//! JSON configuration files using serde
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::prelude::*,
    path::{Path, PathBuf},
};

#[cfg(feature = "json_config")]
use serde_json;
#[cfg(feature = "yaml_config")]
use serde_yaml;
#[cfg(feature = "toml_config")]
use toml;

#[cfg(feature = "proxy")]
use crate::client::data::proxy::ProxyType;

use crate::error::Error::InvalidConfig;
#[cfg(feature = "toml_config")]
use crate::error::TomlError;
use crate::error::{ConfigError, Result};

/// Configuration for IRC clients.
///
/// # Building a configuration programmatically
///
/// For some use cases, it may be useful to build configurations programmatically. Since `Config` is
/// an ordinary struct with public fields, this should be rather straightforward. However, it is
/// important to note that the use of `Config::default()` is important, even when specifying all
/// visible fields because `Config` keeps track of whether it was loaded from a file or
/// programmatically defined, in order to produce better error messages. Using `Config::default()`
/// as below will ensure that this process is handled correctly.
///
/// ```
/// # extern crate irc;
/// use irc::client::prelude::Config;
///
/// # fn main() {
/// let config = Config {
///     nickname: Some("test".to_owned()),
///     server: Some("irc.example.com".to_owned()),
///     ..Config::default()
/// };
/// # }
/// ```
///
/// # Loading a configuration from a file
///
/// The standard method of using a configuration is to load it from a TOML file. You can find an
/// example TOML configuration in the README, as well as a minimal example with code for loading the
/// configuration below.
///
/// ## TOML (`config.toml`)
/// ```toml
/// nickname = "test"
/// server = "irc.example.com"
/// ```
///
/// ## Rust
/// ```no_run
/// # extern crate irc;
/// use irc::client::prelude::Config;
///
/// # fn main() {
/// let config = Config::load("config.toml").unwrap();
/// # }
/// ```
#[derive(Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    /// A list of the owners of the client by nickname (for bots).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub owners: Vec<String>,
    /// The client's nickname.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nickname: Option<String>,
    /// The client's NICKSERV password.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub nick_password: Option<String>,
    /// Alternative nicknames for the client, if the default is taken.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub alt_nicks: Vec<String>,
    /// The client's username.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub username: Option<String>,
    /// The client's real name.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub realname: Option<String>,
    /// The server to connect to.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub server: Option<String>,
    /// The port to connect on.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub port: Option<u16>,
    /// The password to connect to the server.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub password: Option<String>,
    /// The proxy type to connect to.
    #[cfg(feature = "proxy")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub proxy_type: Option<ProxyType>,
    /// The proxy server to connect to.
    #[cfg(feature = "proxy")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub proxy_server: Option<String>,
    /// The proxy port to connect on.
    #[cfg(feature = "proxy")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub proxy_port: Option<u16>,
    /// The username to connect to the proxy server.
    #[cfg(feature = "proxy")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub proxy_username: Option<String>,
    /// The password to connect to the proxy server.
    #[cfg(feature = "proxy")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub proxy_password: Option<String>,
    /// Whether or not to use TLS.
    /// Clients will automatically panic if this is enabled without TLS support.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub use_tls: Option<bool>,
    /// The path to the TLS certificate for this server in DER format.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cert_path: Option<String>,
    /// The path to a TLS certificate to use for CertFP client authentication in DER format.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub client_cert_path: Option<String>,
    /// The password for the certificate to use in CertFP authentication.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub client_cert_pass: Option<String>,
    /// On `true`, all certificate validations are skipped. Defaults to `false`.
    ///
    /// # Warning
    /// You should think very carefully before using this method. If invalid hostnames are trusted, *any* valid
    /// certificate for *any* site will be trusted for use. This introduces significant vulnerabilities, and should
    /// only be used as a last resort.
    pub dangerously_accept_invalid_certs: Option<bool>,
    /// The encoding type used for this connection.
    /// This is typically UTF-8, but could be something else.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub encoding: Option<String>,
    /// A list of channels to join on connection.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub channels: Vec<String>,
    /// User modes to set on connect. Example: "+RB -x"
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub umodes: Option<String>,
    /// The text that'll be sent in response to CTCP USERINFO requests.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub user_info: Option<String>,
    /// The text that'll be sent in response to CTCP VERSION requests.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub version: Option<String>,
    /// The text that'll be sent in response to CTCP SOURCE requests.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub source: Option<String>,
    /// The amount of inactivity in seconds before the client will ping the server.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ping_time: Option<u32>,
    /// The amount of time in seconds for a client to reconnect due to no ping response.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub ping_timeout: Option<u32>,
    /// The length in seconds of a rolling window for message throttling. If more than
    /// `max_messages_in_burst` messages are sent within `burst_window_length` seconds, additional
    /// messages will be delayed automatically as appropriate. In particular, in the past
    /// `burst_window_length` seconds, there will never be more than `max_messages_in_burst` messages
    /// sent.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub burst_window_length: Option<u32>,
    /// The maximum number of messages that can be sent in a burst window before they'll be delayed.
    /// Messages are automatically delayed as appropriate.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub max_messages_in_burst: Option<u32>,
    /// Whether the client should use NickServ GHOST to reclaim its primary nickname if it is in
    /// use. This has no effect if `nick_password` is not set.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub should_ghost: bool,
    /// The command(s) that should be sent to NickServ to recover a nickname. The nickname and
    /// password will be appended in that order after the command.
    /// E.g. `["RECOVER", "RELEASE"]` means `RECOVER nick pass` and `RELEASE nick pass` will be sent
    /// in that order.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub ghost_sequence: Option<Vec<String>>,
    /// Whether or not to use a fake connection for testing purposes. You probably will never want
    /// to enable this, but it is used in unit testing for the `irc` crate.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub use_mock_connection: bool,
    /// The initial value used by the fake connection for testing. You probably will never need to
    /// set this, but it is used in unit testing for the `irc` crate.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub mock_initial_value: Option<String>,

    /// A mapping of channel names to keys for join-on-connect.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "HashMap::is_empty"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub channel_keys: HashMap<String, String>,
    /// A map of additional options to be stored in config.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "HashMap::is_empty"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub options: HashMap<String, String>,

    /// The path that this configuration was loaded from.
    ///
    /// This should not be specified in any configuration. It will automatically be handled by the library.
    #[cfg_attr(feature = "serde", serde(skip_serializing))]
    #[doc(hidden)]
    pub path: Option<PathBuf>,
}

#[cfg(feature = "serde")]
fn is_false(v: &bool) -> bool {
    !v
}

impl Config {
    fn with_path<P: AsRef<Path>>(mut self, path: P) -> Config {
        self.path = Some(path.as_ref().to_owned());
        self
    }

    /// Returns the location this Config was loaded from or `<none>`.
    pub(crate) fn path(&self) -> String {
        self.path
            .as_ref()
            .map(|buf| buf.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<none>".to_owned())
    }

    /// Loads a configuration from the desired path. This will use the file extension to detect
    /// which format to parse the file as (json, toml, or yaml). Using each format requires having
    /// its respective crate feature enabled. Only json is available by default.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let mut file = File::open(&path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        let res = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("json") => Config::load_json(&path, &data),
            Some("toml") => Config::load_toml(&path, &data),
            Some("yaml") | Some("yml") => Config::load_yaml(&path, &data),
            Some(ext) => Err(InvalidConfig {
                path: path.as_ref().to_string_lossy().into_owned(),
                cause: ConfigError::UnknownConfigFormat {
                    format: ext.to_owned(),
                },
            }),
            None => Err(InvalidConfig {
                path: path.as_ref().to_string_lossy().into_owned(),
                cause: ConfigError::MissingExtension,
            }),
        };

        res.map(|config| config.with_path(path))
    }

    #[cfg(feature = "json_config")]
    fn load_json<P: AsRef<Path>>(path: P, data: &str) -> Result<Config> {
        serde_json::from_str(data).map_err(|e| InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::InvalidJson(e),
        })
    }

    #[cfg(not(feature = "json_config"))]
    fn load_json<P: AsRef<Path>>(path: P, _: &str) -> Result<Config> {
        Err(InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::ConfigFormatDisabled { format: "JSON" },
        })
    }

    #[cfg(feature = "toml_config")]
    fn load_toml<P: AsRef<Path>>(path: P, data: &str) -> Result<Config> {
        toml::from_str(data).map_err(|e| InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::InvalidToml(TomlError::Read(e)),
        })
    }

    #[cfg(not(feature = "toml_config"))]
    fn load_toml<P: AsRef<Path>>(path: P, _: &str) -> Result<Config> {
        Err(InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::ConfigFormatDisabled { format: "TOML" },
        })
    }

    #[cfg(feature = "yaml_config")]
    fn load_yaml<P: AsRef<Path>>(path: P, data: &str) -> Result<Config> {
        serde_yaml::from_str(data).map_err(|e| InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::InvalidYaml(e),
        })
    }

    #[cfg(not(feature = "yaml_config"))]
    fn load_yaml<P: AsRef<Path>>(path: P, _: &str) -> Result<Config> {
        Err(InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::ConfigFormatDisabled { format: "YAML" },
        })
    }

    /// Saves a configuration to the desired path. This will use the file extension to detect
    /// which format to parse the file as (json, toml, or yaml). Using each format requires having
    /// its respective crate feature enabled. Only json is available by default.
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let _ = self.path.take();
        let mut file = File::create(&path)?;
        let data = match path.as_ref().extension().and_then(|s| s.to_str()) {
            Some("json") => self.save_json(&path)?,
            Some("toml") => self.save_toml(&path)?,
            Some("yaml") | Some("yml") => self.save_yaml(&path)?,
            Some(ext) => {
                return Err(InvalidConfig {
                    path: path.as_ref().to_string_lossy().into_owned(),
                    cause: ConfigError::UnknownConfigFormat {
                        format: ext.to_owned(),
                    },
                })
            }
            None => {
                return Err(InvalidConfig {
                    path: path.as_ref().to_string_lossy().into_owned(),
                    cause: ConfigError::MissingExtension,
                })
            }
        };
        file.write_all(data.as_bytes())?;
        self.path = Some(path.as_ref().to_owned());
        Ok(())
    }

    #[cfg(feature = "json_config")]
    fn save_json<P: AsRef<Path>>(&self, path: &P) -> Result<String> {
        serde_json::to_string(self).map_err(|e| InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::InvalidJson(e),
        })
    }

    #[cfg(not(feature = "json_config"))]
    fn save_json<P: AsRef<Path>>(&self, path: &P) -> Result<String> {
        Err(InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::ConfigFormatDisabled { format: "JSON" },
        })
    }

    #[cfg(feature = "toml_config")]
    fn save_toml<P: AsRef<Path>>(&self, path: &P) -> Result<String> {
        toml::to_string(self).map_err(|e| InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::InvalidToml(TomlError::Write(e)),
        })
    }

    #[cfg(not(feature = "toml_config"))]
    fn save_toml<P: AsRef<Path>>(&self, path: &P) -> Result<String> {
        Err(InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::ConfigFormatDisabled { format: "TOML" },
        })
    }

    #[cfg(feature = "yaml_config")]
    fn save_yaml<P: AsRef<Path>>(&self, path: &P) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::InvalidYaml(e),
        })
    }

    #[cfg(not(feature = "yaml_config"))]
    fn save_yaml<P: AsRef<Path>>(&self, path: &P) -> Result<String> {
        Err(InvalidConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            cause: ConfigError::ConfigFormatDisabled { format: "YAML" },
        })
    }

    /// Determines whether or not the nickname provided is the owner of the bot.
    pub fn is_owner(&self, nickname: &str) -> bool {
        self.owners.iter().find(|n| *n == nickname).is_some()
    }

    /// Gets the nickname specified in the configuration.
    pub fn nickname(&self) -> Result<&str> {
        self.nickname
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| InvalidConfig {
                path: self.path(),
                cause: ConfigError::NicknameNotSpecified,
            })
    }

    /// Gets the bot's nickserv password specified in the configuration.
    /// This defaults to an empty string when not specified.
    pub fn nick_password(&self) -> &str {
        self.nick_password.as_ref().map_or("", String::as_str)
    }

    /// Gets the alternate nicknames specified in the configuration.
    /// This defaults to an empty vector when not specified.
    pub fn alternate_nicknames(&self) -> &[String] {
        &self.alt_nicks
    }

    /// Gets the username specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    pub fn username(&self) -> &str {
        self.username
            .as_ref()
            .map_or(self.nickname().unwrap_or("user"), |s| &s)
    }

    /// Gets the real name specified in the configuration.
    /// This defaults to the user's nickname when not specified.
    pub fn real_name(&self) -> &str {
        self.realname
            .as_ref()
            .map_or(self.nickname().unwrap_or("irc"), |s| &s)
    }

    /// Gets the address of the server specified in the configuration.
    pub fn server(&self) -> Result<&str> {
        self.server
            .as_ref()
            .map(String::as_str)
            .ok_or_else(|| InvalidConfig {
                path: self.path(),
                cause: ConfigError::ServerNotSpecified,
            })
    }

    /// Gets the port of the server specified in the configuration.
    /// This defaults to 6697 (or 6667 if use_tls is specified as false) when not specified.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    pub fn port(&self) -> u16 {
        self.port.as_ref().cloned().unwrap_or(match self.use_tls() {
            true => 6697,
            false => 6667,
        })
    }

    /// Gets the port of the server specified in the configuration.
    /// This defaults to 6667 when not specified.
    #[cfg(not(any(feature = "tls-native", feature = "tls-rust")))]
    pub fn port(&self) -> u16 {
        self.port.as_ref().cloned().unwrap_or(6667)
    }

    /// Gets the server password specified in the configuration.
    /// This defaults to an empty string when not specified.
    pub fn password(&self) -> &str {
        self.password.as_ref().map_or("", String::as_str)
    }

    /// Gets the type of the proxy specified in the configuration.
    /// This defaults to a None ProxyType when not specified.
    #[cfg(feature = "proxy")]
    pub fn proxy_type(&self) -> ProxyType {
        self.proxy_type.as_ref().cloned().unwrap_or(ProxyType::None)
    }

    /// Gets the address of the proxy specified in the configuration.
    /// This defaults to "localhost" string when not specified.
    #[cfg(feature = "proxy")]
    pub fn proxy_server(&self) -> &str {
        self.proxy_server
            .as_ref()
            .map_or("localhost", String::as_str)
    }

    /// Gets the port of the proxy specified in the configuration.
    /// This defaults to 1080 when not specified.
    #[cfg(feature = "proxy")]
    pub fn proxy_port(&self) -> u16 {
        self.proxy_port.as_ref().cloned().unwrap_or(1080)
    }

    /// Gets the username of the proxy specified in the configuration.
    /// This defaults to an empty string when not specified.
    #[cfg(feature = "proxy")]
    pub fn proxy_username(&self) -> &str {
        self.proxy_username.as_ref().map_or("", String::as_str)
    }

    /// Gets the password of the proxy specified in the configuration.
    /// This defaults to an empty string when not specified.
    #[cfg(feature = "proxy")]
    pub fn proxy_password(&self) -> &str {
        self.proxy_password.as_ref().map_or("", String::as_str)
    }

    /// Gets whether or not to use TLS with this connection.
    /// This defaults to true when not specified.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    pub fn use_tls(&self) -> bool {
        self.use_tls.as_ref().cloned().map_or(true, |s| s)
    }

    /// Gets the path to the TLS certificate in DER format if specified.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    pub fn cert_path(&self) -> Option<&str> {
        self.cert_path.as_ref().map(String::as_str)
    }

    /// Gets whether or not to dangerously accept invalid certificates.
    /// This defaults to `false` when not specified.
    pub fn dangerously_accept_invalid_certs(&self) -> bool {
        self.dangerously_accept_invalid_certs.as_ref().cloned().unwrap_or(false)
    }

    /// Gets the path to the client authentication certificate in DER format if specified.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    pub fn client_cert_path(&self) -> Option<&str> {
        self.client_cert_path.as_ref().map(String::as_str)
    }

    /// Gets the password to the client authentication certificate.
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    pub fn client_cert_pass(&self) -> &str {
        self.client_cert_pass.as_ref().map_or("", String::as_str)
    }

    /// Gets the encoding to use for this connection. This requires the encode feature to work.
    /// This defaults to UTF-8 when not specified.
    pub fn encoding(&self) -> &str {
        self.encoding.as_ref().map_or("UTF-8", |s| &s)
    }

    /// Gets the channels to join upon connection.
    /// This defaults to an empty vector if it's not specified.
    pub fn channels(&self) -> &[String] {
        &self.channels
    }

    /// Gets the key for the specified channel if it exists in the configuration.
    pub fn channel_key(&self, chan: &str) -> Option<&str> {
        self.channel_keys.get(chan).map(String::as_str)
    }

    /// Gets the user modes to set on connect specified in the configuration.
    /// This defaults to an empty string when not specified.
    pub fn umodes(&self) -> &str {
        self.umodes.as_ref().map_or("", String::as_str)
    }

    /// Gets the string to be sent in response to CTCP USERINFO requests.
    /// This defaults to an empty string when not specified.
    pub fn user_info(&self) -> &str {
        self.user_info.as_ref().map_or("", String::as_str)
    }

    /// Gets the string to be sent in response to CTCP VERSION requests.
    /// This defaults to `irc:version:env` when not specified.
    /// For example, `irc:0.12.0:Compiled with rustc`
    pub fn version(&self) -> &str {
        self.version.as_ref().map_or(crate::VERSION_STR, |s| &s)
    }

    /// Gets the string to be sent in response to CTCP SOURCE requests.
    /// This defaults to `https://github.com/aatxe/irc` when not specified.
    pub fn source(&self) -> &str {
        self.source
            .as_ref()
            .map_or("https://github.com/aatxe/irc", String::as_str)
    }

    /// Gets the amount of time in seconds for the interval at which the client pings the server.
    /// This defaults to 180 seconds when not specified.
    pub fn ping_time(&self) -> u32 {
        self.ping_time.as_ref().cloned().unwrap_or(180)
    }

    /// Gets the amount of time in seconds for the client to disconnect after not receiving a ping
    /// response.
    /// This defaults to 20 seconds when not specified.
    pub fn ping_timeout(&self) -> u32 {
        self.ping_timeout.as_ref().cloned().unwrap_or(20)
    }

    /// The amount of time in seconds to consider a window for burst messages. The message throttling
    /// system maintains the invariant that in the past `burst_window_length` seconds, the maximum
    /// number of messages sent is `max_messages_in_burst`.
    /// This defaults to 8 seconds when not specified.
    pub fn burst_window_length(&self) -> u32 {
        self.burst_window_length.as_ref().cloned().unwrap_or(8)
    }

    /// The maximum number of messages that can be sent in a burst window before they'll be delayed.
    /// Messages are automatically delayed until the start of the next window. The message throttling
    /// system maintains the invariant that in the past `burst_window_length` seconds, the maximum
    /// number of messages sent is `max_messages_in_burst`.
    /// This defaults to 15 messages when not specified.
    pub fn max_messages_in_burst(&self) -> u32 {
        self.max_messages_in_burst.as_ref().cloned().unwrap_or(15)
    }

    /// Gets whether or not to attempt nickname reclamation using NickServ GHOST.
    /// This defaults to false when not specified.
    pub fn should_ghost(&self) -> bool {
        self.should_ghost
    }

    /// Gets the NickServ command sequence to recover a nickname.
    /// This defaults to `["GHOST"]` when not specified.
    pub fn ghost_sequence(&self) -> Option<&[String]> {
        self.ghost_sequence.as_ref().map(Vec::as_slice)
    }

    /// Looks up the specified string in the options map.
    pub fn get_option(&self, option: &str) -> Option<&str> {
        self.options.get(option).map(String::as_str)
    }

    /// Gets whether or not to use a mock connection for testing.
    /// This defaults to false when not specified.
    pub fn use_mock_connection(&self) -> bool {
        self.use_mock_connection
    }

    /// Gets the initial value for the mock connection.
    /// This defaults to false when not specified.
    /// This has no effect if `use_mock_connection` is not `true`.
    pub fn mock_initial_value(&self) -> &str {
        self.mock_initial_value.as_ref().map_or("", |s| &s)
    }
}

#[cfg(test)]
mod test {
    use super::Config;
    use std::collections::HashMap;

    #[cfg(any(
        feature = "json_config",
        feature = "toml_config",
        feature = "yaml_config"
    ))]
    use super::Result;

    #[allow(unused)]
    fn test_config() -> Config {
        Config {
            owners: vec![format!("test")],
            nickname: Some(format!("test")),
            username: Some(format!("test")),
            realname: Some(format!("test")),
            password: Some(String::new()),
            umodes: Some(format!("+BR")),
            server: Some(format!("irc.test.net")),
            port: Some(6667),
            encoding: Some(format!("UTF-8")),
            channels: vec![format!("#test"), format!("#test2")],

            ..Default::default()
        }
    }

    #[test]
    fn is_owner() {
        let cfg = Config {
            owners: vec![format!("test"), format!("test2")],
            ..Default::default()
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
                map
            },
            ..Default::default()
        };
        assert_eq!(cfg.get_option("testing"), Some("test"));
        assert_eq!(cfg.get_option("not"), None);
    }

    #[test]
    #[cfg(feature = "json_config")]
    fn load_from_json() -> Result<()> {
        const DATA: &str = include_str!("client_config.json");
        assert_eq!(
            Config::load_json("client_config.json", DATA)?.with_path("client_config.json"),
            test_config().with_path("client_config.json")
        );
        Ok(())
    }

    #[test]
    #[cfg(feature = "toml_config")]
    fn load_from_toml() -> Result<()> {
        const DATA: &str = include_str!("client_config.toml");
        assert_eq!(
            Config::load_toml("client_config.toml", DATA)?.with_path("client_config.toml"),
            test_config().with_path("client_config.toml")
        );
        Ok(())
    }

    #[test]
    #[cfg(feature = "yaml_config")]
    fn load_from_yaml() -> Result<()> {
        const DATA: &str = include_str!("client_config.yaml");
        assert_eq!(
            Config::load_yaml("client_config.yaml", DATA)?.with_path("client_config.yaml"),
            test_config().with_path("client_config.yaml")
        );
        Ok(())
    }
}
