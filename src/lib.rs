//! A simple, thread-safe IRC client library.
#![crate_name = "irc"]
#![crate_type = "lib"]
#![unstable]

#![feature(if_let)]
#![feature(slicing_syntax)]
extern crate serialize;

pub mod conn;
pub mod data;
pub mod server;
