//! A simple, thread-safe IRC library.
#![crate_name = "irc"]
#![crate_type = "lib"]
#![unstable]
#![warn(missing_docs)]

#![feature(collections, core, io, slice_patterns, str_char, tcp)]
#[cfg(feature = "ctcp")] extern crate time;
#[cfg(feature = "encode")] extern crate encoding;
extern crate rustc_serialize;
#[cfg(feature = "ssl")] extern crate openssl;

pub mod client;
pub mod server;
