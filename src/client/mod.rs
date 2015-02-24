//! A simple, thread-safe IRC client library.
#![stable]

pub mod conn;
pub mod data;
pub mod server;

pub mod prelude {
    //! A client-side IRC prelude, re-exporting all the necessary basics.
    #![unstable = "Prelude is newly added, and contents have not yet firmed up."]

    pub use client::server::{IrcServer, Server};
    pub use client::server::utils::Wrapper;
    pub use client::data::{Command, Config, Message, Response, ToMessage};
    pub use client::data::kinds::{IrcReader, IrcWriter};
}
