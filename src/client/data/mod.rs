//! Data related to IRC functionality.

pub use proto::caps::{Capability, NegotiationVersion};
pub use proto::command::Command;
pub use client::data::config::Config;
pub use proto::message::Message;
pub use proto::response::Response;
pub use client::data::user::{AccessLevel, User};

pub mod kinds {
    //! Trait definitions of appropriate Writers and Buffers for use with this library.
    use std::io::prelude::*;

    /// Trait describing all possible Writers for this library.
    pub trait IrcWrite: Write + Sized + Send + 'static {}
    impl<T> IrcWrite for T where T: Write + Sized + Send + 'static {}

    /// Trait describing all possible Readers for this library.
    pub trait IrcRead: BufRead + Sized + Send + 'static {}
    impl<T> IrcRead for T where T: BufRead + Sized + Send + 'static {}
}

pub mod config;
pub mod user;
