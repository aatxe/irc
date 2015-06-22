//! A simple, thread-safe IRC client library.

pub mod conn;
pub mod data;
pub mod server;

pub mod prelude {
    //! A client-side IRC prelude, re-exporting all the necessary basics.
    pub use client::server::{IrcServer, Server};
    pub use client::server::utils::ServerExt;
    pub use client::data::{Capability, Command, Config, Message, NegotiationVersion, Response};
    pub use client::data::kinds::{IrcRead, IrcWrite};
}

#[cfg(test)]
pub mod test {
    use std::io::{BufReader, Empty, empty};

    pub fn buf_empty() -> BufReader<Empty> {
        BufReader::new(empty())
    }
}
