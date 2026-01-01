//! SASL authentication support
//!
//! ```
//! use irc::client::prelude::Config;
//! use irc::client::data::sasl::SASLMode;
//!
//! # fn main() {
//! let config = Config {
//!     nickname: Some("test".to_owned()),
//!     server: Some("irc.example.com".to_owned()),
//!     login: Some("server_login".to_owned()),
//!     password: Some("server_password".to_owned()),
//!     sasl: Some(SASLMode::Plain),
//!     ..Config::default()
//! };
//! # }
//! ```
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// An enum which defines which type of SASL authentication mode should be used.
#[derive(Clone, PartialEq, Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SASLMode {
    /// Do not use any SASL auth
    None,
    /// Authenticate in SASL PLAIN mode
    Plain,
}
