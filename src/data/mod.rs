pub mod kinds {
    pub trait IrcWriter: Writer + Sized + Send + 'static {}
    impl<T> IrcWriter for T where T: Writer + Sized + Send + 'static {}
    pub trait IrcReader: Buffer + Sized + Send + 'static {}
    impl<T> IrcReader for T where T: Buffer + Sized + Send + 'static {}
}

pub mod command;
pub mod config;
pub mod message;
