//! A simple, thread-safe IRC client library.
#![crate_name = "irc"]
#![crate_type = "lib"]
#![unstable]
#![warn(missing_docs)]

#![feature(associated_types, slicing_syntax)]
#[cfg(feature = "ctcp")] extern crate time;
#[cfg(feature = "encode")] extern crate encoding;
extern crate "rustc-serialize" as rustc_serialize;
#[cfg(feature = "ssl")] extern crate openssl;

pub mod conn;
pub mod data;
pub mod server;
