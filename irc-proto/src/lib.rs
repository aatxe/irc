//! Support for the IRC protocol using Tokio.

#![warn(missing_docs)]

extern crate bufstream;
extern crate bytes;
extern crate chrono;
#[macro_use]
extern crate failure;
extern crate encoding;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate log;
extern crate native_tls;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "json")]
extern crate serde_json;
#[cfg(feature = "yaml")]
extern crate serde_yaml;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_mockstream;
extern crate tokio_timer;
extern crate tokio_tls;
#[cfg(feature = "toml")]
extern crate toml;

pub mod caps;
pub mod chan;
pub mod command;
pub mod irc;
pub mod line;
pub mod message;
pub mod mode;
pub mod response;

pub use self::caps::{Capability, NegotiationVersion};
pub use self::chan::ChannelExt;
pub use self::command::{BatchSubCommand, CapSubCommand, Command};
pub use self::irc::IrcCodec;
pub use self::message::Message;
pub use self::mode::{ChannelMode, Mode, UserMode};
pub use self::response::Response;
