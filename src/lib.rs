//! A simple, thread-safe, and async-friendly library for IRC clients.
//!
//! # Quick Start
//! The main public API is entirely exported in [`client::prelude`](./client/prelude/index.html).
//! This should include everything necessary to write an IRC client or bot.
//!
//! # A Whirlwind Tour
//! The irc crate is divided into two main modules: [`client`](./client/index.html) and
//! [`proto`](./proto/index.html). As the names suggest, the `client` module captures the whole of
//! the client-side functionality, while the `proto` module features general components of an IRC
//! protocol implementation that could in principle be used in either client or server software.
//! Both modules feature a number of components that are low-level and can be used to build
//! alternative APIs for the IRC protocol. For the average user, the higher-level components for an
//! IRC client are all re-exported in [`client::prelude`](./client/prelude/index.html). That module
//! serves as the best starting point for a new user trying to understand the high-level API.
//!
//! # Example
//!
//! ```no_run
//! # extern crate irc;
//! use irc::client::prelude::*;
//!
//! # fn main() {
//! // configuration is loaded from config.toml into a Config
//! let client = IrcClient::new("config.toml").unwrap();
//! // identify comes from ClientExt
//! client.identify().unwrap();
//! // for_each_incoming comes from Client
//! client.for_each_incoming(|irc_msg| {
//!     // irc_msg is a Message
//!     if let Command::PRIVMSG(channel, message) = irc_msg.command {
//!         if message.contains(&*client.current_nickname()) {
//!             // send_privmsg comes from ClientExt
//!             client.send_privmsg(&channel, "beep boop").unwrap();
//!         }
//!     }
//! }).unwrap();
//! # }
//! ```

#![warn(missing_docs)]

extern crate bufstream;
extern crate bytes;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate encoding;
#[macro_use]
extern crate futures;
pub extern crate irc_proto as proto;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "json")]
extern crate serde_json;
#[cfg(feature = "yaml")]
extern crate serde_yaml;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_mockstream;
extern crate tokio_timer;
extern crate tokio_tls;
#[cfg(feature = "toml")]
extern crate toml;

pub mod client;
pub mod error;

const VERSION_STR: &str = concat!(
    env!("CARGO_PKG_NAME"),
    ":",
    env!("CARGO_PKG_VERSION"),
    ":Compiled with rustc",
);
