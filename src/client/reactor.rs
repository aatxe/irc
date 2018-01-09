//! A system for creating and managing multiple connections to IRC servers.

use client::data::Config;
use client::server::{IrcServer, IrcServerFuture, Server};
use error;
use proto::Message;

use futures::{Future, Stream};
use futures::future;
use tokio_core::reactor::Core;

pub struct IrcReactor {
    inner: Core,
    handlers: Vec<Box<Future<Item = (), Error = error::Error>>>,
}

impl IrcReactor {
    pub fn new() -> error::Result<IrcReactor> {
        Ok(IrcReactor {
            inner: Core::new()?,
            handlers: Vec::new(),
        })
    }

    pub fn prepare_server<'a>(&mut self, config: &'a Config) -> error::Result<IrcServerFuture<'a>> {
        IrcServer::new_future(self.inner.handle(), config)
    }

    pub fn connect_server(&mut self, future: IrcServerFuture) -> error::Result<IrcServer> {
        self.inner.run(future)
    }

    pub fn prepare_server_and_connect(&mut self, config: &Config) -> error::Result<IrcServer> {
        self.prepare_server(config).and_then(|future| self.connect_server(future))
    }

    pub fn register_server_with_handler<F>(
        &mut self, server: IrcServer, handler: F
    ) where F: Fn(&IrcServer, Message) -> error::Result<()> + 'static  {
        self.handlers.push(Box::new(server.stream().for_each(move |message| {
            handler(&server, message)
        })));
    }

    pub fn run(&mut self) -> error::Result<()> {
        let mut handlers = Vec::new();
        while let Some(handler) = self.handlers.pop() {
            handlers.push(handler);
        }
        self.inner.run(future::join_all(handlers).map(|_| ()))
    }
}
