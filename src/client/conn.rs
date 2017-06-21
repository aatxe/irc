//! A module providing IRC connections for use by `IrcServer`s.
use std::fmt;
use error;
use client::data::Config;
use client::transport::IrcTransport;
use proto::{IrcCodec, Message};
use futures::{Async, Poll, Future, Sink, StartSend, Stream};
use native_tls::TlsConnector;
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_io::AsyncRead;
use tokio_tls::{TlsConnectorExt, TlsStream};

/// An IRC connection used internally by `IrcServer`.
pub enum Connection {
    #[doc(hidden)]
    Unsecured(IrcTransport<TcpStream>),
    #[doc(hidden)]
    Secured(IrcTransport<TlsStream<TcpStream>>),
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Connection::Unsecured(_) => "Connection::Unsecured(...)",
                Connection::Secured(_) => "Connection::Secured(...)",
            }
        )
    }
}

/// A convenient type alias representing the TlsStream future.
type TlsFuture = Box<Future<Error = error::Error, Item = TlsStream<TcpStream>> + Send>;

/// A future representing an eventual `Connection`.
pub enum ConnectionFuture<'a> {
    #[doc(hidden)]
    Unsecured(&'a Config, TcpStreamNew),
    #[doc(hidden)]
    Secured(&'a Config, TlsFuture),
}

impl<'a> Future for ConnectionFuture<'a> {
    type Item = Connection;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self {
            &mut ConnectionFuture::Unsecured(ref config, ref mut inner) => {
                let framed = try_ready!(inner.poll()).framed(IrcCodec::new(config.encoding())?);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Unsecured(transport)))
            }
            &mut ConnectionFuture::Secured(ref config, ref mut inner) => {
                let framed = try_ready!(inner.poll()).framed(IrcCodec::new(config.encoding())?);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Secured(transport)))
            }
        }
    }
}

impl Connection {
    /// Creates a new `Connection` using the specified `Config` and `Handle`.
    pub fn new<'a>(config: &'a Config, handle: &Handle) -> error::Result<ConnectionFuture<'a>> {
        if config.use_ssl() {
            let domain = format!("{}:{}", config.server(), config.port());
            let connector = TlsConnector::builder()?.build()?;
            let stream = TcpStream::connect(&config.socket_addr(), handle)
                .map_err(|e| {
                    let res: error::Error = e.into();
                    res
                })
                .and_then(move |socket| {
                    connector.connect_async(&domain, socket).map_err(
                        |e| e.into(),
                    )
                })
                .boxed();
            Ok(ConnectionFuture::Secured(config, stream))
        } else {
            Ok(ConnectionFuture::Unsecured(
                config,
                TcpStream::connect(&config.socket_addr(), handle),
            ))
        }
    }
}

impl Stream for Connection {
    type Item = Message;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self {
            &mut Connection::Unsecured(ref mut inner) => inner.poll(),
            &mut Connection::Secured(ref mut inner) => inner.poll(),
        }
    }
}

impl Sink for Connection {
    type SinkItem = Message;
    type SinkError = error::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match self {
            &mut Connection::Unsecured(ref mut inner) => inner.start_send(item),
            &mut Connection::Secured(ref mut inner) => inner.start_send(item),
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match self {
            &mut Connection::Unsecured(ref mut inner) => inner.poll_complete(),
            &mut Connection::Secured(ref mut inner) => inner.poll_complete(),
        }
    }
}
