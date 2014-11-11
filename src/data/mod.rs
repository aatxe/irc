//! Data related to IRC functionality.
#![unstable]

pub use data::command::Command;
pub use data::config::Config;
pub use data::message::Message;
pub use data::user::User;

pub mod kinds {
    //! Trait definitions of appropriate Writers and Buffers for use with this library.
    #![unstable]

    /// Trait describing all possible Writers for this library.
    #[unstable]
    pub trait IrcWriter: Writer + Sized + Send + 'static {}
    impl<T> IrcWriter for T where T: Writer + Sized + Send + 'static {}
    /// Trait describing all possible Readers for this library.
    #[unstable]
    pub trait IrcReader: Buffer + Sized + Send + 'static {}
    impl<T> IrcReader for T where T: Buffer + Sized + Send + 'static {}
    /// Trait describing all possible Streams for this library.
    #[unstable]
    pub trait IrcStream: IrcWriter + IrcReader {}
    impl<T> IrcStream for T where T: IrcWriter + IrcReader {}
}

pub mod command;
pub mod config;
pub mod message;
pub mod user;
