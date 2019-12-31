//! A module providing IRC connections for use by `IrcServer`s.
use futures_channel::mpsc::UnboundedSender;
use futures_util::{sink::Sink, stream::Stream};
use native_tls::{Certificate, Identity, TlsConnector};
use std::{
    fmt,
    fs::File,
    io::Read,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::net::TcpStream;
use tokio_tls::{self, TlsStream};
use tokio_util::codec::Decoder;

use crate::{
    client::{
        data::Config,
        transport::{LogView, Logged, Transport},
    },
    error,
    proto::{IrcCodec, Message},
};

/// An IRC connection used internally by `IrcServer`.
pub enum Connection {
    #[doc(hidden)]
    Unsecured(Transport<TcpStream>),
    #[doc(hidden)]
    Secured(Transport<TlsStream<TcpStream>>),
    #[doc(hidden)]
    Mock(Logged<crate::client::mock::MockStream>),
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

impl Connection {
    /// Creates a new `Connection` using the specified `Config`
    pub(crate) async fn new(
        config: &Config,
        tx: UnboundedSender<Message>,
    ) -> error::Result<Connection> {
        if config.use_mock_connection() {
            use encoding::{label::encoding_from_whatwg_label, EncoderTrap};

            let encoding = encoding_from_whatwg_label(config.encoding()).ok_or_else(|| {
                error::Error::UnknownCodec {
                    codec: config.encoding().to_owned(),
                }
            })?;

            let init_str = config.mock_initial_value();
            let initial = encoding
                .encode(init_str, EncoderTrap::Replace)
                .map_err(|data| error::Error::CodecFailed {
                    codec: encoding.name(),
                    data: data.into_owned(),
                })?;

            let stream = crate::client::mock::MockStream::new(&initial);
            let framed = IrcCodec::new(config.encoding())?.framed(stream);
            let transport = Transport::new(&config, framed, tx);
            return Ok(Connection::Mock(Logged::wrap(transport)));
        }

        if config.use_ssl() {
            let domain = format!("{}", config.server()?);
            log::info!("Connecting via SSL to {}.", domain);
            let mut builder = TlsConnector::builder();

            if let Some(cert_path) = config.cert_path() {
                let mut file = File::open(cert_path)?;
                let mut cert_data = vec![];
                file.read_to_end(&mut cert_data)?;
                let cert = Certificate::from_der(&cert_data)?;
                builder.add_root_certificate(cert);
                log::info!("Added {} to trusted certificates.", cert_path);
            }

            if let Some(client_cert_path) = config.client_cert_path() {
                let client_cert_pass = config.client_cert_pass();
                let mut file = File::open(client_cert_path)?;
                let mut client_cert_data = vec![];
                file.read_to_end(&mut client_cert_data)?;
                let pkcs12_archive = Identity::from_pkcs12(&client_cert_data, &client_cert_pass)?;
                builder.identity(pkcs12_archive);
                log::info!(
                    "Using {} for client certificate authentication.",
                    client_cert_path
                );
            }
            let connector: tokio_tls::TlsConnector = builder.build()?.into();

            let socket = TcpStream::connect(config.to_socket_addrs()?).await?;
            let stream = connector.connect(&domain, socket).await?;
            let framed = IrcCodec::new(config.encoding())?.framed(stream);
            let transport = Transport::new(&config, framed, tx);

            Ok(Connection::Secured(transport))
        } else {
            log::info!("Connecting to {}.", config.server()?);
            let stream = TcpStream::connect(config.to_socket_addrs()?).await?;
            let framed = IrcCodec::new(config.encoding())?.framed(stream);
            let transport = Transport::new(&config, framed, tx);

            Ok(Connection::Unsecured(transport))
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
    type Item = error::Result<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut *self {
            Connection::Unsecured(inner) => Pin::new(inner).poll_next(cx),
            Connection::Secured(inner) => Pin::new(inner).poll_next(cx),
            Connection::Mock(inner) => Pin::new(inner).poll_next(cx),
        }
    }
}

impl Sink<Message> for Connection {
    type Error = error::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut *self {
            Connection::Unsecured(inner) => Pin::new(inner).poll_ready(cx),
            Connection::Secured(inner) => Pin::new(inner).poll_ready(cx),
            Connection::Mock(inner) => Pin::new(inner).poll_ready(cx),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match &mut *self {
            Connection::Unsecured(inner) => Pin::new(inner).start_send(item),
            Connection::Secured(inner) => Pin::new(inner).start_send(item),
            Connection::Mock(inner) => Pin::new(inner).start_send(item),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut *self {
            Connection::Unsecured(inner) => Pin::new(inner).poll_flush(cx),
            Connection::Secured(inner) => Pin::new(inner).poll_flush(cx),
            Connection::Mock(inner) => Pin::new(inner).poll_flush(cx),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut *self {
            Connection::Unsecured(inner) => Pin::new(inner).poll_close(cx),
            Connection::Secured(inner) => Pin::new(inner).poll_close(cx),
            Connection::Mock(inner) => Pin::new(inner).poll_close(cx),
        }
    }
}
