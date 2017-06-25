//! A simple, thread-safe, and async-friendly IRC client library.

pub mod conn;
pub mod data;
pub mod server;
pub mod transport;

pub mod prelude {
    //! A client-side IRC prelude, re-exporting all the necessary basics.
    pub use client::data::Config;
    pub use client::server::{IrcServer, Server};
    pub use client::server::utils::ServerExt;
    pub use proto::{Capability, ChannelExt, Command, Message, NegotiationVersion, Response};
    pub use proto::{ChannelMode, Mode, UserMode};

    pub use futures::{Future, Stream};
}
