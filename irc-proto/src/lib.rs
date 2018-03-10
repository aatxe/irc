//! Support for the IRC protocol using Tokio.

#![warn(missing_docs)]

extern crate bytes;
extern crate encoding;
#[macro_use]
extern crate failure;
extern crate tokio_io;

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
