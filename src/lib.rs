//! A simple, thread-safe IRC library.

#![warn(missing_docs)]

extern crate time;
extern crate encoding;
extern crate native_tls;
extern crate rustc_serialize;
extern crate tokio_core;
extern crate tokio_service;
extern crate tokio_tls;

pub mod client;
pub mod proto;
pub mod server;
