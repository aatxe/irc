//! A simple, thread-safe IRC client library.
#![crate_name = "irc"]
#![crate_type = "lib"]
#![license = "Public Domain"]
#![unstable]

#![feature(if_let, slicing_syntax)]
extern crate serialize;
#[cfg(feature = "ssl")] extern crate openssl;

pub mod conn;
pub mod data;
pub mod server;
