//! A simple, thread-safe IRC client library.
#![crate_name = "irc"]
#![crate_type = "lib"]

#![feature(if_let)]
#![feature(phase)]
#![feature(slicing_syntax)]
extern crate regex;
#[phase(plugin)] extern crate regex_macros;
extern crate serialize;

mod conn;
pub mod server;
mod utils;
