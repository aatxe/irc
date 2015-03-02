//! Data related to IRC functionality.
#![stable]

pub use client::data::command::Command;
pub use client::data::config::Config;
pub use client::data::message::{Message, ToMessage};
pub use client::data::response::Response;
pub use client::data::user::{AccessLevel, User};

pub mod kinds {
    //! Trait definitions of appropriate Writers and Buffers for use with this library.
    #![stable]
    use std::io::prelude::*;

    /// Trait describing all possible Writers for this library.
    #[stable]
    pub trait IrcWrite: Write + Sized + Send + 'static {}
    impl<T> IrcWrite for T where T: Write + Sized + Send + 'static {}
    /// Trait describing all possible Readers for this library.
    #[stable]
    pub trait IrcRead: BufRead + Sized + Send + 'static {}
    impl<T> IrcRead for T where T: BufRead + Sized + Send + 'static {}
}

pub mod command;
pub mod config;
pub mod message;
pub mod response;
pub mod user;
