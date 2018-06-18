//! A module providing IRC connections for use by `IrcServer`s.
use std::fs::File;
use std::fmt;
use std::io::Read;

use encoding::EncoderTrap;
use encoding::label::encoding_from_whatwg_label;
use futures::future;
use futures::{Async, Poll, Future, Sink, StartSend, Stream};
use native_tls::{Certificate, TlsConnector, Pkcs12};
use tokio::net::{ConnectFuture, TcpStream};
use tokio_io::AsyncRead;
use tokio_mockstream::MockStream;
use tokio_tls::{TlsConnectorExt, TlsStream};

use error;
use client::data::Config;
use client::transport::{IrcTransport, LogView, Logged};
use proto::{IrcCodec, Message};

/// An IRC connection used internally by `IrcServer`.
pub enum Connection {
    #[doc(hidden)]
    Unsecured(IrcTransport<TcpStream>),
    #[doc(hidden)]
    Secured(IrcTransport<TlsStream<TcpStream>>),
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

type TlsFuture = Box<Future<Error = error::IrcError, Item = TlsStream<TcpStream>> + Send>;

/// A future representing an eventual `Connection`.
pub enum ConnectionFuture {
    #[doc(hidden)]
    Unsecured(Config, ConnectFuture),
    #[doc(hidden)]
    Secured(Config, TlsFuture),
    #[doc(hidden)]
    Mock(Config),
}

impl fmt::Debug for ConnectionFuture {
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
                ConnectionFuture::Unsecured(ref cfg, _) |
                ConnectionFuture::Secured(ref cfg, _) |
                ConnectionFuture::Mock(ref cfg) => cfg,
            }
        )
    }
}

impl Future for ConnectionFuture {
    type Item = Connection;
    type Error = error::IrcError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            ConnectionFuture::Unsecured(ref config, ref mut inner) => {
                let framed = try_ready!(inner.poll()).framed(IrcCodec::new(config.encoding())?);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Unsecured(transport)))
            }
            ConnectionFuture::Secured(ref config, ref mut inner) => {
                let framed = try_ready!(inner.poll()).framed(IrcCodec::new(config.encoding())?);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Secured(transport)))
            }
            ConnectionFuture::Mock(ref config) => {
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

                let framed = MockStream::new(&initial?).framed(IrcCodec::new(config.encoding())?);
                let transport = IrcTransport::new(config, framed);

                Ok(Async::Ready(Connection::Mock(Logged::wrap(transport))))
            }
        }
    }
}

impl Connection {
    /// Creates a future yielding a new `Connection` using the specified `Config`.
    pub fn new(config: Config) -> impl Future<Item = Connection, Error = error::IrcError> {
        future::lazy(move || Connection::new_inner(config))
            .and_then(|connection_future| connection_future)
    }

    fn new_inner(config: Config) -> error::Result<ConnectionFuture> {
        if config.use_mock_connection() {
            Ok(ConnectionFuture::Mock(config))
        } else if config.use_ssl() {
            let domain = format!("{}", config.server()?);
            info!("Connecting via SSL to {}.", domain);
            let mut builder = TlsConnector::builder()?;

            if let Some(cert_path) = config.cert_path() {
                let mut file = File::open(cert_path)?;
                let mut cert_data = vec![];
                file.read_to_end(&mut cert_data)?;
                let cert = Certificate::from_der(&cert_data)?;
                builder.add_root_certificate(cert)?;
                info!("Added {} to trusted certificates.", cert_path);
            }

            if let Some(client_cert_path) = config.client_cert_path() {
                let client_cert_pass = config.client_cert_pass();
                let mut file = File::open(client_cert_path)?;
                let mut client_cert_data = vec![];
                file.read_to_end(&mut client_cert_data)?;
                let pkcs12_archive = Pkcs12::from_der(&client_cert_data, &client_cert_pass)?;
                builder.identity(pkcs12_archive)?;
                info!("Using {} for client certificate authentication.", client_cert_path);
            }

            let connector = builder.build()?;
            let socket_addr = config.socket_addr()?;

            let stream = TcpStream::connect(&socket_addr).map_err(|e| {
                let res: error::IrcError = e.into();
                res
            }).and_then(move |socket| {
                connector.connect_async(&domain, socket).map_err(|e| e.into())
            });

            Ok(ConnectionFuture::Secured(config, Box::new(stream)))
        } else {
            info!("Connecting to {}.", config.server()?);
            let socket_addr = config.socket_addr()?;
            Ok(ConnectionFuture::Unsecured(
                config,
                TcpStream::connect(&socket_addr),
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
