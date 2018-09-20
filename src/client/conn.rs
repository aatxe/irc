//! A module providing IRC connections for use by `IrcServer`s.
use std::fmt;

use encoding::EncoderTrap;
use encoding::label::encoding_from_whatwg_label;
use futures::{Async, Poll, Future, Sink, StartSend, Stream};
use tokio_codec::Decoder;
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_mockstream::MockStream;
use tokio_rustls::{self, TlsStream, rustls, webpki};

use error;
use client::data::Config;
use client::transport::{IrcTransport, LogView, Logged};
use proto::{IrcCodec, Message};

/// An IRC connection used internally by `IrcServer`.
pub enum Connection {
    #[doc(hidden)]
    Unsecured(IrcTransport<TcpStream>),
    #[doc(hidden)]
    Secured(IrcTransport<RustlsStream>),
    #[doc(hidden)]
    Mock(Logged<MockStream>),
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Connection::Unsecured(_) => "Connection::Unsecured(...)",
                Connection::Secured(_) => "Connection::Secured(...)",
                Connection::Mock(_) => "Connection::Mock(...)",
            }
        )
    }
}

/// A type alias for the appropriate `tokio_rustls::TlsStream` type.
type RustlsStream = TlsStream<TcpStream, rustls::ClientSession>;

/// A convenient type alias representing the `TlsStream` future.
type TlsFuture = Box<Future<Error = error::IrcError, Item = RustlsStream> + Send>;

/// A future representing an eventual `Connection`.
pub enum ConnectionFuture<'a> {
    #[doc(hidden)]
    Unsecured(&'a Config, TcpStreamNew),
    #[doc(hidden)]
    Secured(&'a Config, TlsFuture),
    #[doc(hidden)]
    Mock(&'a Config),
}

impl<'a> fmt::Debug for ConnectionFuture<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({:?}, ...)",
            match *self {
                ConnectionFuture::Unsecured(_, _) => "ConnectionFuture::Unsecured",
                ConnectionFuture::Secured(_, _) => "ConnectionFuture::Secured",
                ConnectionFuture::Mock(_) => "ConnectionFuture::Mock",
            },
            match *self {
                ConnectionFuture::Unsecured(cfg, _) |
                ConnectionFuture::Secured(cfg, _) |
                ConnectionFuture::Mock(cfg) => cfg,
            }
        )
    }
}

impl<'a> Future for ConnectionFuture<'a> {
    type Item = Connection;
    type Error = error::IrcError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            ConnectionFuture::Unsecured(config, ref mut inner) => {
                let stream = try_ready!(inner.poll());
                let framed = IrcCodec::new(config.encoding())?.framed(stream);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Unsecured(transport)))
            }
            ConnectionFuture::Secured(config, ref mut inner) => {
                let stream = try_ready!(inner.poll());
                let framed = IrcCodec::new(config.encoding())?.framed(stream);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Secured(transport)))
            }
            ConnectionFuture::Mock(config) => {
                let enc: error::Result<_> = encoding_from_whatwg_label(
                    config.encoding()
                ).ok_or_else(|| error::IrcError::UnknownCodec {
                    codec: config.encoding().to_owned(),
                });
                let encoding = enc?;
                let init_str = config.mock_initial_value();
                let initial: error::Result<_> = {
                    encoding.encode(init_str, EncoderTrap::Replace).map_err(|data| {
                        error::IrcError::CodecFailed {
                            codec: encoding.name(),
                            data: data.into_owned(),
                        }
                    })
                };

                let stream = MockStream::new(&initial?);
                let framed = IrcCodec::new(config.encoding())?.framed(stream);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Mock(Logged::wrap(transport))))
            }
        }
    }
}

impl Connection {
    /// Creates a new `Connection` using the specified `Config` and `Handle`.
    pub fn new<'a>(config: &'a Config, handle: &Handle) -> error::Result<ConnectionFuture<'a>> {
        if config.use_mock_connection() {
            Ok(ConnectionFuture::Mock(config))
        } else if config.use_ssl() {
            let domain = config.server()?;
            info!("Connecting via SSL to {}.", domain);
            let domain = webpki::DNSNameRef::try_from_ascii_str(domain).map_err(|()| {
                error::IrcError::DomainNameSyntaxError { input: domain.to_owned() }
            })?.to_owned();
            let connector: tokio_rustls::TlsConnector = config.rustls_config()?.into();
            let stream = Box::new(TcpStream::connect(&config.socket_addr()?, handle).map_err(|e| {
                let res: error::IrcError = e.into();
                res
            }).and_then(move |socket| {
                connector.connect(domain.as_ref(), socket).map_err(
                    |e| e.into(),
                )
            }));
            Ok(ConnectionFuture::Secured(config, stream))
        } else {
            info!("Connecting to {}.", config.server()?);
            Ok(ConnectionFuture::Unsecured(
                config,
                TcpStream::connect(&config.socket_addr()?, handle),
            ))
        }
    }

    /// Gets a view of the internal logging if and only if this connection is using a mock stream.
    /// Otherwise, this will always return `None`. This is used for unit testing.
    pub fn log_view(&self) -> Option<LogView> {
        match *self {
            Connection::Mock(ref inner) => Some(inner.view()),
            _ => None,
        }
    }
}

impl Stream for Connection {
    type Item = Message;
    type Error = error::IrcError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match *self {
            Connection::Unsecured(ref mut inner) => inner.poll(),
            Connection::Secured(ref mut inner) => inner.poll(),
            Connection::Mock(ref mut inner) => inner.poll(),
        }
    }
}

impl Sink for Connection {
    type SinkItem = Message;
    type SinkError = error::IrcError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match *self {
            Connection::Unsecured(ref mut inner) => inner.start_send(item),
            Connection::Secured(ref mut inner) => inner.start_send(item),
            Connection::Mock(ref mut inner) => inner.start_send(item),
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match *self {
            Connection::Unsecured(ref mut inner) => inner.poll_complete(),
            Connection::Secured(ref mut inner) => inner.poll_complete(),
            Connection::Mock(ref mut inner) => inner.poll_complete(),
        }
    }
}
