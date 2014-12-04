//! A simple, thread-safe IRC client library.
#![crate_name = "irc"]
#![crate_type = "lib"]
#![unstable]
#![warn(missing_docs)]

#![feature(if_let, slicing_syntax)]
#[cfg(feature = "encode")] extern crate encoding;
extern crate serialize;
#[cfg(feature = "ssl")] extern crate openssl;

pub mod conn;
pub mod data;
pub mod server;
