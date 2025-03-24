//! A module providing IRC connections for use by `IrcServer`s.
use futures_util::{sink::Sink, stream::Stream};
use pin_project::pin_project;
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::codec::Framed;

#[cfg(feature = "proxy")]
use tokio_socks::tcp::Socks5Stream;

#[cfg(feature = "proxy")]
use crate::client::data::ProxyType;

#[cfg(all(feature = "tls-native", not(feature = "tls-rust")))]
use std::{fs::File, io::Read};

#[cfg(all(feature = "tls-native", not(feature = "tls-rust")))]
use native_tls::{Certificate, Identity, TlsConnector};

#[cfg(all(feature = "tls-native", not(feature = "tls-rust")))]
use tokio_native_tls::{self, TlsStream};

#[cfg(feature = "tls-rust")]
use std::{
    convert::TryFrom,
    fs::File,
    io::{BufReader, Error, ErrorKind},
    sync::Arc,
};
#[cfg(feature = "tls-rust")]
use tokio_rustls::client::TlsStream;
#[cfg(feature = "tls-rust")]
use tokio_rustls::{
    rustls::client::danger::{ServerCertVerified, ServerCertVerifier},
    rustls::crypto::{verify_tls12_signature, verify_tls13_signature, CryptoProvider},
    rustls::pki_types::{
        CertificateDer as Certificate, PrivateKeyDer as PrivateKey, ServerName, UnixTime,
    },
    rustls::{self, ClientConfig, RootCertStore},
    TlsConnector,
};

use crate::{
    client::{
        data::Config,
        mock::MockStream,
        transport::{LogView, Logged, Transport},
    },
    error,
    proto::{IrcCodec, Message},
};

/// An IRC connection used internally by `IrcServer`.
#[pin_project(project = ConnectionProj)]
pub enum Connection {
    #[doc(hidden)]
    Unsecured(#[pin] Transport<TcpStream>),
    #[doc(hidden)]
    #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
    Secured(#[pin] Transport<TlsStream<TcpStream>>),
    #[doc(hidden)]
    Mock(#[pin] Logged<MockStream>),
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Connection::Unsecured(_) => "Connection::Unsecured(...)",
                #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
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
            log::info!("Connecting via mock to {}.", config.server()?);
            return Ok(Connection::Mock(Logged::wrap(
                Self::new_mocked_transport(config, tx).await?,
            )));
        }

        #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
        {
            if config.use_tls() {
                log::info!("Connecting via TLS to {}.", config.server()?);
                return Ok(Connection::Secured(
                    Self::new_secured_transport(config, tx).await?,
                ));
            }
        }

        log::info!("Connecting to {}.", config.server()?);
        Ok(Connection::Unsecured(
            Self::new_unsecured_transport(config, tx).await?,
        ))
    }

    #[cfg(not(feature = "proxy"))]
    async fn new_stream(config: &Config) -> error::Result<TcpStream> {
        Ok(TcpStream::connect((config.server()?, config.port())).await?)
    }

    #[cfg(feature = "proxy")]
    async fn new_stream(config: &Config) -> error::Result<TcpStream> {
        let server = config.server()?;
        let port = config.port();
        let address = (server, port);

        match config.proxy_type() {
            ProxyType::None => Ok(TcpStream::connect(address).await?),
            ProxyType::Socks5 => {
                let proxy_server = config.proxy_server();
                let proxy_port = config.proxy_port();
                let proxy = (proxy_server, proxy_port);

                log::info!("Setup proxy {:?}.", proxy);

                let proxy_username = config.proxy_username();
                let proxy_password = config.proxy_password();
                if !proxy_username.is_empty() || !proxy_password.is_empty() {
                    return Ok(Socks5Stream::connect_with_password(
                        proxy,
                        address,
                        proxy_username,
                        proxy_password,
                    )
                    .await?
                    .into_inner());
                }

                Ok(Socks5Stream::connect(proxy, address).await?.into_inner())
            }
        }
    }

    async fn new_unsecured_transport(
        config: &Config,
        tx: UnboundedSender<Message>,
    ) -> error::Result<Transport<TcpStream>> {
        let stream = Self::new_stream(config).await?;
        let framed = Framed::new(stream, IrcCodec::new(config.encoding())?);

        Ok(Transport::new(config, framed, tx))
    }

    #[cfg(all(feature = "tls-native", not(feature = "tls-rust")))]
    async fn new_secured_transport(
        config: &Config,
        tx: UnboundedSender<Message>,
    ) -> error::Result<Transport<TlsStream<TcpStream>>> {
        let mut builder = TlsConnector::builder();

        if let Some(cert_path) = config.cert_path() {
            if let Ok(mut file) = File::open(cert_path) {
                let mut cert_data = vec![];
                file.read_to_end(&mut cert_data)?;
                let cert = Certificate::from_der(&cert_data)?;
                builder.add_root_certificate(cert);
                log::info!("Added {} to trusted certificates.", cert_path);
            } else {
                return Err(error::Error::InvalidConfig {
                    path: config.path(),
                    cause: error::ConfigError::FileMissing {
                        file: cert_path.to_string(),
                    },
                });
            }
        }

        if let Some(client_cert_path) = config.client_cert_path() {
            if let Ok(mut file) = File::open(client_cert_path) {
                let mut client_cert_data = vec![];
                file.read_to_end(&mut client_cert_data)?;
                let client_cert_pass = config.client_cert_pass();
                let pkcs12_archive = Identity::from_pkcs12(&client_cert_data, client_cert_pass)?;
                builder.identity(pkcs12_archive);
                log::info!(
                    "Using {} for client certificate authentication.",
                    client_cert_path
                );
            } else {
                return Err(error::Error::InvalidConfig {
                    path: config.path(),
                    cause: error::ConfigError::FileMissing {
                        file: client_cert_path.to_string(),
                    },
                });
            }
        }

        if config.dangerously_accept_invalid_certs() {
            builder.danger_accept_invalid_certs(true);
        }

        let connector: tokio_native_tls::TlsConnector = builder.build()?.into();
        let domain = config.server()?;

        let stream = Self::new_stream(config).await?;
        let stream = connector.connect(domain, stream).await?;
        let framed = Framed::new(stream, IrcCodec::new(config.encoding())?);

        Ok(Transport::new(config, framed, tx))
    }

    #[cfg(feature = "tls-rust")]
    async fn new_secured_transport(
        config: &Config,
        tx: UnboundedSender<Message>,
    ) -> error::Result<Transport<TlsStream<TcpStream>>> {
        #[derive(Debug)]
        struct DangerousAcceptAllVerifier(Arc<CryptoProvider>);

        impl DangerousAcceptAllVerifier {
            fn new() -> Self {
                DangerousAcceptAllVerifier(CryptoProvider::get_default()
                    .expect("no process default crypto provider has been set - application must call CryptoProvider::install_default()")
                    .clone())
            }
        }

        impl ServerCertVerifier for DangerousAcceptAllVerifier {
            fn verify_server_cert(
                &self,
                _end_entity: &Certificate,
                _intermediates: &[Certificate],
                _server_name: &ServerName,
                _oscp: &[u8],
                _now: UnixTime,
            ) -> Result<ServerCertVerified, rustls::Error> {
                return Ok(ServerCertVerified::assertion());
            }

            fn verify_tls12_signature(
                &self,
                message: &[u8],
                cert: &Certificate<'_>,
                dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                verify_tls12_signature(
                    message,
                    cert,
                    dss,
                    &self.0.signature_verification_algorithms,
                )
            }

            fn verify_tls13_signature(
                &self,
                message: &[u8],
                cert: &Certificate<'_>,
                dss: &rustls::DigitallySignedStruct,
            ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
            {
                verify_tls13_signature(
                    message,
                    cert,
                    dss,
                    &self.0.signature_verification_algorithms,
                )
            }

            fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
                self.0.signature_verification_algorithms.supported_schemes()
            }
        }

        enum ClientAuth {
            SingleCert(Vec<Certificate<'static>>, PrivateKey<'static>),
            NoClientAuth,
        }

        let client_auth = if let Some(client_cert_path) = config.client_cert_path() {
            if let Ok(file) = File::open(client_cert_path) {
                let client_cert_data =
                    rustls_pemfile::certs(&mut BufReader::new(file)).collect::<Result<_, _>>()?;

                let client_cert_pass = config.client_cert_pass();
                let client_cert_pass = rustls_pemfile::private_key(
                    &mut client_cert_pass.as_bytes(),
                )?
                .ok_or_else(|| error::Error::InvalidConfig {
                    path: config.path(),
                    cause: error::ConfigError::UnknownConfigFormat {
                        format: "Failed to parse private key".to_string(),
                    },
                })?;

                log::info!(
                    "Using {} for client certificate authentication.",
                    client_cert_path
                );

                ClientAuth::SingleCert(client_cert_data, client_cert_pass)
            } else {
                return Err(error::Error::InvalidConfig {
                    path: config.path(),
                    cause: error::ConfigError::FileMissing {
                        file: client_cert_path.to_string(),
                    },
                });
            }
        } else {
            ClientAuth::NoClientAuth
        };

        macro_rules! make_client_auth {
            ($builder:expr) => {
                match client_auth {
                    ClientAuth::SingleCert(data, pass) => {
                        $builder.with_client_auth_cert(data, pass).map_err(|err| {
                            error::Error::Io(Error::new(ErrorKind::InvalidInput, err))
                        })?
                    }
                    ClientAuth::NoClientAuth => $builder.with_no_client_auth(),
                }
            };
        }

        let builder = ClientConfig::builder();

        let tls_config = if config.dangerously_accept_invalid_certs() {
            let builder = builder
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(DangerousAcceptAllVerifier::new()));
            make_client_auth!(builder)
        } else {
            let mut root_store = RootCertStore::empty();

            #[cfg(feature = "webpki-roots")]
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let native_certs = rustls_native_certs::load_native_certs();
            for cert in native_certs.certs {
                root_store.add(cert.into())?;
            }

            if let Some(cert_path) = config.cert_path() {
                if let Ok(file) = File::open(cert_path) {
                    let certificates = rustls_pemfile::certs(&mut BufReader::new(file))
                        .collect::<Result<Vec<_>, _>>()?;
                    let (added, ignored) = root_store.add_parsable_certificates(certificates);

                    if ignored > 0 {
                        log::warn!("Failed to parse some certificates in {}", cert_path);
                    }

                    if added > 0 {
                        log::info!("Added {} to trusted certificates.", cert_path);
                    }
                } else {
                    return Err(error::Error::InvalidConfig {
                        path: config.path(),
                        cause: error::ConfigError::FileMissing {
                            file: cert_path.to_string(),
                        },
                    });
                }
            }

            let builder = builder.with_root_certificates(root_store);
            make_client_auth!(builder)
        };

        let connector = TlsConnector::from(Arc::new(tls_config));
        let domain = ServerName::try_from(config.server()?)?.to_owned();
        let stream = Self::new_stream(config).await?;
        let stream = connector.connect(domain, stream).await?;
        let framed = Framed::new(stream, IrcCodec::new(config.encoding())?);

        Ok(Transport::new(config, framed, tx))
    }

    async fn new_mocked_transport(
        config: &Config,
        tx: UnboundedSender<Message>,
    ) -> error::Result<Transport<MockStream>> {
        let init_str = config.mock_initial_value();

        let initial = {
            #[cfg(feature = "encoding")]
            {
                use encoding::{label::encoding_from_whatwg_label, EncoderTrap};

                let encoding = encoding_from_whatwg_label(config.encoding()).ok_or_else(|| {
                    error::Error::UnknownCodec {
                        codec: config.encoding().to_owned(),
                    }
                })?;
                encoding
                    .encode(init_str, EncoderTrap::Replace)
                    .map_err(|data| error::Error::CodecFailed {
                        codec: encoding.name(),
                        data: data.into_owned(),
                    })?
            }

            #[cfg(not(feature = "encoding"))]
            {
                init_str.as_bytes()
            }
        };
        let stream = MockStream::new(&initial);
        let framed = Framed::new(stream, IrcCodec::new(config.encoding())?);

        Ok(Transport::new(config, framed, tx))
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

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project() {
            ConnectionProj::Unsecured(inner) => inner.poll_next(cx),
            #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
            ConnectionProj::Secured(inner) => inner.poll_next(cx),
            ConnectionProj::Mock(inner) => inner.poll_next(cx),
        }
    }
}

impl Sink<Message> for Connection {
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            ConnectionProj::Unsecured(inner) => inner.poll_ready(cx),
            #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
            ConnectionProj::Secured(inner) => inner.poll_ready(cx),
            ConnectionProj::Mock(inner) => inner.poll_ready(cx),
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match self.project() {
            ConnectionProj::Unsecured(inner) => inner.start_send(item),
            #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
            ConnectionProj::Secured(inner) => inner.start_send(item),
            ConnectionProj::Mock(inner) => inner.start_send(item),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            ConnectionProj::Unsecured(inner) => inner.poll_flush(cx),
            #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
            ConnectionProj::Secured(inner) => inner.poll_flush(cx),
            ConnectionProj::Mock(inner) => inner.poll_flush(cx),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.project() {
            ConnectionProj::Unsecured(inner) => inner.poll_close(cx),
            #[cfg(any(feature = "tls-native", feature = "tls-rust"))]
            ConnectionProj::Secured(inner) => inner.poll_close(cx),
            ConnectionProj::Mock(inner) => inner.poll_close(cx),
        }
    }
}
