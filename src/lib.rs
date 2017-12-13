//! A simple, thread-safe, and async-friendly library for IRC clients.

#![warn(missing_docs)]
#![recursion_limit="128"]

extern crate bufstream;
extern crate bytes;
extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate encoding;
#[macro_use]
extern crate futures;
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
pub mod proto;

const VERSION_STR: &'static str = concat!(
    env!("CARGO_PKG_NAME"),
    ":",
    env!("CARGO_PKG_VERSION"),
    ":Compiled with rustc",
);
