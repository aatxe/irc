//! A simple, thread-safe IRC library.

#![warn(missing_docs)]

extern crate bufstream;
extern crate bytes;
extern crate encoding;
extern crate futures;
extern crate native_tls;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_service;
extern crate tokio_tls;

pub mod client;
pub mod proto;
pub mod server;
