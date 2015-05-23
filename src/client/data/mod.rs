//! Data related to IRC functionality.

pub use client::data::caps::{Capability, NegotiationVersion};
pub use client::data::command::Command;
pub use client::data::config::Config;
pub use client::data::message::Message;
pub use client::data::response::Response;
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

pub mod caps;
pub mod command;
pub mod config;
pub mod message;
pub mod response;
pub mod user;
