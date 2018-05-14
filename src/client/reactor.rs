//! A system for creating and managing IRC client connections.
//!
//! This API provides the ability to create and manage multiple IRC clients that can run on the same
//! thread through the use of a shared event loop. It can also be used to encapsulate the dependency
//! on `tokio` and `futures` in the use of `IrcClient::new_future`. This means that knowledge of
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
//!   let client = reactor.prepare_client_and_connect(&config).unwrap();
//!   reactor.register_client_with_handler(client, process_msg);
//!   reactor.run().unwrap();
//! }
//! # fn process_msg(client: &IrcClient, message: Message) -> error::Result<()> { Ok(()) }
//! ```

use futures::{Future, IntoFuture, Stream};
use futures::future;
use tokio_core::reactor::{Core, Handle};

use client::data::Config;
use client::{IrcClient, IrcClientFuture, PackedIrcClient, Client};
use error;
use proto::Message;

/// A thin wrapper over an event loop.
///
/// An IRC reactor is used to create new IRC clients and to drive the management of all connected
/// clients as the application runs. It can be used to run multiple clients on the same thread, as
/// well as to get better control over error management in an IRC client.
///
/// For a full example usage, see [`irc::client::reactor`](./index.html).
pub struct IrcReactor {
    inner: Core,
    handlers: Vec<Box<Future<Item = (), Error = error::IrcError>>>,
}

impl IrcReactor {
    /// Creates a new reactor.
    pub fn new() -> error::Result<IrcReactor> {
        Ok(IrcReactor {
            inner: Core::new()?,
            handlers: Vec::new(),
        })
    }

    /// Creates a representation of an IRC client that has not yet attempted to connect. In
    /// particular, this representation is as a `Future` that when run will produce a connected
    /// [`IrcClient`](../struct.IrcClient.html).
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let future_client = IrcReactor::new().and_then(|mut reactor| {
    ///     reactor.prepare_client(&config)
    /// });
    /// # }
    /// ```
    pub fn prepare_client<'a>(&mut self, config: &'a Config) -> error::Result<IrcClientFuture<'a>> {
        IrcClient::new_future(self.inner_handle(), config)
    }

    /// Runs an [`IrcClientFuture`](../struct.IrcClientFuture.html), such as one from
    /// `prepare_client` to completion, yielding an [`IrcClient`](../struct.IrcClient.html).
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let client = IrcReactor::new().and_then(|mut reactor| {
    ///     reactor.prepare_client(&config).and_then(|future| {
    ///         reactor.connect_client(future)
    ///     })
    /// });
    /// # }
    /// ```
    pub fn connect_client(&mut self, future: IrcClientFuture) -> error::Result<IrcClient> {
        self.inner.run(future).map(|PackedIrcClient(client, future)| {
            self.register_future(future);
            client
        })
    }

    /// Creates a new [`IrcClient`](../struct.IrcClient.html) from the specified configuration,
    /// connecting immediately. This is guaranteed to be the composition of `prepare_client` and
    /// `connect_client`.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let client = IrcReactor::new().and_then(|mut reactor| {
    ///     reactor.prepare_client_and_connect(&config)
    /// });
    /// # }
    /// ```
    pub fn prepare_client_and_connect(&mut self, config: &Config) -> error::Result<IrcClient> {
        self.prepare_client(config).and_then(|future| self.connect_client(future))
    }

    /// Registers the given client with the specified message handler. The reactor will store this
    /// setup until the next call to run, where it will be used to process new messages over the
    /// connection indefinitely (or until failure). As registration is consumed by `run`, subsequent
    /// calls to run will require new registration.
    ///
    /// **Note**: A client can only be registered once. Subsequent attempts will cause a panic.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let config = Config::default();
    /// let mut reactor = IrcReactor::new().unwrap();
    /// let client = reactor.prepare_client_and_connect(&config).unwrap();
    /// reactor.register_client_with_handler(client, |client, msg| {
    ///   // Message processing happens here.
    ///   Ok(())
    /// })
    /// # }
    /// ```
    pub fn register_client_with_handler<F, U>(
        &mut self, client: IrcClient, mut handler: F
    ) where F: FnMut(&IrcClient, Message) -> U + 'static,
            U: IntoFuture<Item = (), Error = error::IrcError> + 'static {
        self.handlers.push(Box::new(client.stream().for_each(move |message| {
            handler(&client, message)
        })));
    }

    /// Registers an arbitrary future with this reactor. This is a sort of escape hatch that allows
    /// you to take more control over what runs on the reactor without requiring you to bring in
    /// additional knowledge about `tokio`. It is suspected that `register_client_with_handler` will
    /// be sufficient for most use cases.
    pub fn register_future<F>(
        &mut self, future: F
    ) where F: IntoFuture<Item = (), Error = error::IrcError> + 'static {
        self.handlers.push(Box::new(future.into_future()))
    }

    /// Returns a handle to the internal event loop. This is a sort of escape hatch that allows you
    /// to take more control over what runs on the reactor using `tokio`. This can be used for
    /// sharing this reactor with some elements of other libraries.
    pub fn inner_handle(&self) -> Handle {
        self.inner.handle()
    }

    /// Consumes all registered handlers and futures, and runs them. When using
    /// `register_client_with_handler`, this will block indefinitely (until failure occurs) as it
    /// will simply continue to process new, incoming messages for each client that was registered.
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
    /// let client = reactor.prepare_client_and_connect(&config).unwrap();
    /// reactor.register_client_with_handler(client, process_msg);
    /// reactor.run().unwrap();
    /// # }
    /// # fn process_msg(client: &IrcClient, message: Message) -> error::Result<()> { Ok(()) }
    /// ```
    pub fn run(&mut self) -> error::Result<()> {
        let mut handlers = Vec::new();
        while let Some(handler) = self.handlers.pop() {
            handlers.push(handler);
        }
        self.inner.run(future::join_all(handlers).map(|_| ()))
    }
}
