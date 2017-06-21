use std::fmt;
use std::thread;
use std::thread::JoinHandle;
use error;
use client::data::Config;
use client::transport::IrcTransport;
use proto::{IrcCodec, Message};
use futures::future;
use futures::{Async, Poll, Future, Sink, StartSend, Stream};
use futures::stream::SplitStream;
use futures::sync::mpsc;
use futures::sync::oneshot;
use futures::sync::mpsc::UnboundedSender;
use native_tls::TlsConnector;
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_io::AsyncRead;
use tokio_tls::{TlsConnectorExt, TlsStream};

pub enum Connection {
    Unsecured(IrcTransport<TcpStream>),
    Secured(IrcTransport<TlsStream<TcpStream>>),
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IrcConnection")
    }
}

type TlsFuture = Box<Future<Error=error::Error, Item=TlsStream<TcpStream>> + Send>;

pub enum ConnectionFuture<'a> {
    Unsecured(&'a Config, TcpStreamNew),
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
    pub fn new<'a>(config: &'a Config, handle: &Handle) -> error::Result<ConnectionFuture<'a>> {
        if config.use_ssl() {
            let domain = format!("{}:{}", config.server(), config.port());
            let connector = TlsConnector::builder()?.build()?;
            let stream = TcpStream::connect(&config.socket_addr(), handle).map_err(|e| {
                let res: error::Error = e.into();
                res
            }).and_then(move |socket| {
                connector.connect_async(&domain, socket).map_err(|e| e.into())
            }).boxed();
            Ok(ConnectionFuture::Secured(config, stream))
        } else {
            Ok(ConnectionFuture::Unsecured(config, TcpStream::connect(&config.socket_addr(), handle)))
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
