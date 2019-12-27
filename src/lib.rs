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
//! use irc::client::prelude::*;
//! use futures::prelude::*;
//!
//! # #[tokio::main]
//! # async fn main() -> irc::error::Result<()> {
//! // configuration is loaded from config.toml into a Config
//! let mut client = Client::new("config.toml").await?;
//! // identify comes from ClientExt
//! client.identify()?;
//!
//! let mut stream = client.stream()?;
//!
//! while let Some(message) = stream.next().await.transpose()? {
//!     if let Command::PRIVMSG(channel, message) = message.command {
//!         if message.contains(&*client.current_nickname()) {
//!             // send_privmsg comes from ClientExt
//!             client.send_privmsg(&channel, "beep boop").unwrap();
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

#[macro_use]
extern crate failure;

pub extern crate irc_proto as proto;

pub mod client;
pub mod error;

const VERSION_STR: &str = concat!(
    env!("CARGO_PKG_NAME"),
    ":",
    env!("CARGO_PKG_VERSION"),
    ":Compiled with rustc",
);
