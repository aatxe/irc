//! Data related to IRC functionality.

pub use crate::client::data::config::Config;
#[cfg(feature = "proxy")]
pub use crate::client::data::proxy::ProxyType;
pub use crate::client::data::user::{AccessLevel, User};

pub mod config;
#[cfg(feature = "proxy")]
pub mod proxy;
pub mod user;
