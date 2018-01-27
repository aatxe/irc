//! A system for creating and managing IRC server connections.
//!
//! This API provides the ability to create and manage multiple IRC servers that can run on the same
//! thread through the use of a shared event loop. It can also be used to encapsulate the dependency
//! on `tokio` and `futures` in the use of `IrcServer::new_future`. This means that knowledge of
//! those libraries should be unnecessary for the average user. Nevertheless, this API also provides
//! some escape hatches that let advanced users take further advantage of these dependencies.
//! 
//! # Example
//! ```no_run
//! # extern crate irc;
//! # use std::default::Default;
//! use irc::client::prelude::*;
//! use irc::error;
//!
//! fn main() {
//!   let config = Config::default();
//!   let mut reactor = IrcReactor::new().unwrap();
//!   let server = reactor.prepare_server_and_connect(&config).unwrap();
//!   reactor.register_server_with_handler(server, process_msg);
//!   reactor.run().unwrap();
//! }
//! # fn process_msg(server: &IrcServer, message: Message) -> error::Result<()> { Ok(()) }
//! ```

use futures::{Future, IntoFuture, Stream};
use futures::future;
use tokio_core::reactor::{Core, Handle};

use client::data::Config;
use client::server::{IrcServer, IrcServerFuture, PackedIrcServer, Server};
use error;
use proto::Message;

/// A thin wrapper over an event loop.
///
/// An IRC reactor is used to create new connections to IRC servers and to drive the management of
/// all connected servers as the application runs. It can be used to run multiple servers on the
/// same thread, as well as to get better control over error management in an IRC client.
///
/// For a full example usage, see [irc::client::reactor](./index.html).
pub struct IrcReactor {
    inner: Core,
    handlers: Vec<Box<Future<Item = (), Error = error::Error>>>,
}

impl IrcReactor {
    /// Creates a new reactor.
    pub fn new() -> error::Result<IrcReactor> {
        Ok(IrcReactor {
            inner: Core::new()?,
            handlers: Vec::new(),
        })
    }

    /// Creates a representation of an IRC server that has not yet attempted to connect. In
    /// particular, this representation is as a Future that when run will produce a connected
    /// [IrcServer](./server/struct.IrcServer.html).
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let future_server = IrcReactor::new().and_then(|mut reactor| {
    ///     reactor.prepare_server(&config)
    /// });
    /// # }
    /// ```
    pub fn prepare_server<'a>(&mut self, config: &'a Config) -> error::Result<IrcServerFuture<'a>> {
        IrcServer::new_future(self.inner_handle(), config)
    }

    /// Runs an [IrcServerFuture](./server/struct.IrcServerFuture.html), such as one from
    /// `prepare_server` to completion, yielding an [IrcServer](./server/struct.IrcServer.html).
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let server = IrcReactor::new().and_then(|mut reactor| {
    ///     reactor.prepare_server(&config).and_then(|future| {
    ///         reactor.connect_server(future)
    ///     })
    /// });
    /// # }
    /// ```
    pub fn connect_server(&mut self, future: IrcServerFuture) -> error::Result<IrcServer> {
        self.inner.run(future).map(|PackedIrcServer(server, future)| {
            self.register_future(future);
            server
        })
    }

    /// Creates a new IRC server from the specified configuration, connecting immediately. This is
    /// guaranteed to be the composition of prepare_server and connect_server.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let server = IrcReactor::new().and_then(|mut reactor| {
    ///     reactor.prepare_server_and_connect(&config)
    /// });
    /// # }
    /// ```
    pub fn prepare_server_and_connect(&mut self, config: &Config) -> error::Result<IrcServer> {
        self.prepare_server(config).and_then(|future| self.connect_server(future))
    }

    /// Registers the given server with the specified message handler. The reactor will store this
    /// setup until the next call to run, where it will be used to process new messages over the
    /// connection indefinitely (or until failure). As registration is consumed by `run`, subsequent
    /// calls to run will require new registration.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let mut reactor = IrcReactor::new().unwrap();
    /// let server = reactor.prepare_server_and_connect(&config).unwrap();
    /// reactor.register_server_with_handler(server, |server, msg| {
    ///   // Message processing happens here.
    ///   Ok(())
    /// })
    /// # }
    /// ```
    pub fn register_server_with_handler<F, U>(
        &mut self, server: IrcServer, handler: F
    ) where F: Fn(&IrcServer, Message) -> U + 'static,
            U: IntoFuture<Item = (), Error = error::Error> + 'static {
        self.handlers.push(Box::new(server.stream().for_each(move |message| {
            handler(&server, message)
        })));
    }

    /// Registers an arbitrary future with this reactor. This is a sort of escape hatch that allows
    /// you to take more control over what runs on the reactor without requiring you to bring in
    /// additional knowledge about `tokio`. It is suspected that `register_server_with_handler` will
    /// be sufficient for most use cases.
    pub fn register_future<F>(
        &mut self, future: F
    ) where F: IntoFuture<Item = (), Error = error::Error> + 'static {
        self.handlers.push(Box::new(future.into_future()))
    }

    /// Returns a handle to the internal event loop. This is a sort of escape hatch that allows you
    /// to take more control over what runs on the reactor using `tokio`. This can be used for
    /// sharing this reactor with some elements of other libraries.
    pub fn inner_handle(&self) -> Handle {
        self.inner.handle()
    }

    /// Consumes all registered handlers and futures, and runs them. When using
    /// `register_server_with_handler`, this will block indefinitely (until failure occurs) as it
    /// will simply continue to process new, incoming messages for each server that was registered.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # use irc::error;
    /// # fn main() {
    /// # let config = Config::default();
    /// let mut reactor = IrcReactor::new().unwrap();
    /// let server = reactor.prepare_server_and_connect(&config).unwrap();
    /// reactor.register_server_with_handler(server, process_msg)
    /// # }
    /// # fn process_msg(server: &IrcServer, message: Message) -> error::Result<()> { Ok(()) }
    /// ```
    pub fn run(&mut self) -> error::Result<()> {
        let mut handlers = Vec::new();
        while let Some(handler) = self.handlers.pop() {
            handlers.push(handler);
        }
        self.inner.run(future::join_all(handlers).map(|_| ()))
    }
}
