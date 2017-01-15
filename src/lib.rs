//! A simple, thread-safe IRC library.

#![warn(missing_docs)]

extern crate time;
extern crate encoding;
extern crate rustc_serialize;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

pub mod client;
pub mod proto;
pub mod server;
