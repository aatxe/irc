//! A simple, thread-safe IRC library.

#![warn(missing_docs)]

extern crate time;
extern crate bufstream;
#[cfg(feature = "encode")] extern crate encoding;
extern crate rustc_serialize;
#[cfg(feature = "ssl")] extern crate openssl;

pub mod client;
pub mod server;
