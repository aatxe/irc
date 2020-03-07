//! A feature which allow us to connect to IRC via a proxy.
//!
//! ```
//! use irc::client::prelude::Config;
//! use irc::client::data::ProxyType;
//!
//! # fn main() {
//! let config = Config {
//!     nickname: Some("test".to_owned()),
//!     server: Some("irc.example.com".to_owned()),
//!     proxy_type: Some(ProxyType::Socks5),
//!     proxy_server: Some("127.0.0.1".to_owned()),
//!     proxy_port: Some(9050),
//!     ..Config::default()
//! };
//! # }
//! ```

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An enum which defines which type of proxy should be in use.
#[cfg(feature = "proxy")]
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ProxyType {
    /// Does not use any proxy.
    None,

    /// Use a SOCKS5 proxy.
    /// DNS queries are also sent via the proxy.
    Socks5,
}
