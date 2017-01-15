//! A simple, thread-safe IRC library.

#![warn(missing_docs)]

extern crate time;
extern crate encoding;
extern crate rustc_serialize;

pub mod client;
pub mod proto;
pub mod server;
