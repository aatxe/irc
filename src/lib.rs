//! A simple, thread-safe IRC library.

#![warn(missing_docs)]

extern crate time;
#[cfg(feature = "encode")] extern crate encoding;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
#[cfg(feature = "ssl")] extern crate openssl;

pub mod client;
pub mod server;
