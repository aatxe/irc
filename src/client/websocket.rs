//! A websocket connection

use std::{
    convert::TryFrom,
    future::Future,
    io,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use pin_project::pin_project;
#[cfg(all(feature = "tls-native", not(feature = "tls-rust")))]
use tokio_native_tls::{self, TlsStream};

#[cfg(feature = "tls-rust")]
use tokio_rustls::client::TlsStream;

use futures_util::{
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
    FutureExt,
};
use proto::{IrcCodec, Message as IrcMessage};
use tokio::{
    net::TcpStream,
    sync::mpsc::UnboundedSender,
    time::{self, Interval, Sleep},
};
use tokio_tungstenite::{
    client_async_with_config,
    tungstenite::{
        self, client::IntoClientRequest, http, protocol::WebSocketConfig, Message, Utf8Bytes,
    },
    WebSocketStream,
};
use tokio_util::{
    bytes::{self, BytesMut},
    codec::{Decoder, Encoder},
    either::Either,
};

use crate::{
    client::{conn::Connection, data::Config, transport::Pinger},
    error,
};

const BINARY_SUBPROTOCOL: &str = "binary.ircv3.net";
const TEXT_SUBPROTOCOL: &str = "text.ircv3.net";
const REQUESTED_SUBPROTOCOLS: &str = "binary.ircv3.net, text.ircv3.net";
const MAX_IRC_WEBSOCKET_MESSAGE_SIZE: usize = 8192;

/// A websocket connection
#[pin_project]
pub struct WebSocketConnection {
    stream: WebSocketStream<Either<TlsStream<TcpStream>, TcpStream>>,
    codec: IrcCodec,
    read: BytesMut,
    mode: Mode,

    #[pin]
    pinger: Option<Pinger>,
    #[pin]
    websocket_pinger: Option<WebsocketPinger>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Binary,
    Text,
}

#[pin_project]
struct WebsocketPinger {
    ping_timeout: Duration,

    #[pin]
    ping_deadline: Option<Sleep>,

    #[pin]
    ping_interval: Interval,
}

enum PingerMessage {
    Ping,
}

impl WebSocketConnection {
    /// Connects to the websocket
    pub async fn connect(config: &Config, tx: UnboundedSender<IrcMessage>) -> error::Result<Self> {
        let (scheme, stream) = if config.use_tls() {
            (
                "wss",
                Either::Left(Connection::new_secured_stream(config).await?),
            )
        } else {
            ("ws", Either::Right(Connection::new_stream(config).await?))
        };

        let mut request = websocket_uri(
            scheme,
            config.server()?,
            config.port(),
            config.websocket_path(),
        )
        .into_client_request()?;
        request.headers_mut().insert(
            http::header::SEC_WEBSOCKET_PROTOCOL,
            http::HeaderValue::from_static(REQUESTED_SUBPROTOCOLS),
        );

        let websocket_config = WebSocketConfig::default()
            .max_message_size(Some(MAX_IRC_WEBSOCKET_MESSAGE_SIZE))
            .max_frame_size(Some(MAX_IRC_WEBSOCKET_MESSAGE_SIZE));

        let (stream, response) =
            client_async_with_config(request, stream, Some(websocket_config)).await?;

        Ok(Self {
            stream,
            codec: IrcCodec::new(config.encoding())?,
            read: BytesMut::new(),
            mode: mode_from_response(&response)?,
            pinger: Some(Pinger::new(tx, config)),
            websocket_pinger: Some(WebsocketPinger::new(config)),
        })
    }

    fn try_send_ping(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Result<(), error::Error> {
        let pinged = if let Some(pinger) = self.as_mut().project().websocket_pinger.as_pin_mut() {
            match pinger.poll(cx) {
                Poll::Ready(Ok(PingerMessage::Ping)) => {
                    log::trace!("Sending websocket ping");

                    let mut fut = self
                        .as_mut()
                        .project()
                        .stream
                        .send(Message::Ping(bytes::Bytes::new()));

                    loop {
                        match fut.poll_unpin(cx) {
                            Poll::Ready(res) => {
                                res?;
                                break;
                            }
                            Poll::Pending => {}
                        }
                    }
                    true
                }
                Poll::Ready(res) => {
                    res?;
                    false
                }
                Poll::Pending => false,
            }
        } else {
            false
        };

        if pinged {
            if let Some(pinger) = self.as_mut().project().websocket_pinger.as_pin_mut() {
                let mut this = pinger.project();
                if this.ping_deadline.is_none() {
                    let ping_deadline: Sleep = time::sleep(*this.ping_timeout);
                    this.ping_deadline.set(Some(ping_deadline));
                }
            }
        }

        Ok(())
    }

    // websocket frames don't include CRLF, so we re-add it for our decoder
    fn push_crlf_line(read: &mut BytesMut, bytes: &[u8]) {
        read.extend_from_slice(bytes);
        if !read.ends_with(b"\r\n") {
            read.extend_from_slice(b"\r\n");
        }
    }
}

impl Stream for WebSocketConnection {
    type Item = Result<IrcMessage, error::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().try_send_ping(cx)?;

        if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
            match pinger.poll(cx) {
                Poll::Ready(result) => result?,
                Poll::Pending => (),
            }
        }

        loop {
            let this = self.as_mut().project();
            if let Some(message) = this.codec.decode(this.read)? {
                if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
                    pinger.handle_message(&message)?;
                }

                return Poll::Ready(Some(Ok(message)));
            }

            match this.stream.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(Message::Text(text)))) => {
                    Self::push_crlf_line(this.read, text.as_ref());
                }
                Poll::Ready(Some(Ok(Message::Binary(bytes)))) => {
                    Self::push_crlf_line(this.read, &bytes);
                }
                Poll::Ready(Some(Ok(Message::Close(_)))) => {
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Ok(Message::Ping(_)))) => {
                    log::trace!("Received websocket ping");

                    let mut fut = self
                        .as_mut()
                        .project()
                        .stream
                        .send(Message::Pong(bytes::Bytes::new()));
                    loop {
                        match fut.poll_unpin(cx) {
                            Poll::Ready(res) => {
                                res?;
                                break;
                            }
                            Poll::Pending => {}
                        }
                    }
                }
                Poll::Ready(Some(Ok(Message::Pong(_)))) => {
                    log::trace!("Received websocket pong");

                    if let Some(mut pinger) = this.websocket_pinger.as_pin_mut() {
                        pinger.as_mut().project().ping_deadline.set(None);
                    }
                }
                Poll::Ready(Some(Ok(_))) => {}
                Poll::Ready(Some(Err(error))) => {
                    return Poll::Ready(Some(Err(websocket_error(error))));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl Sink<IrcMessage> for WebSocketConnection {
    type Error = error::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .stream
            .poll_ready_unpin(cx)
            .map_err(websocket_error)
    }

    fn start_send(mut self: Pin<&mut Self>, item: IrcMessage) -> Result<(), Self::Error> {
        let this = self.as_mut().project();
        let mut bytes = BytesMut::new();

        this.codec.encode(item, &mut bytes)?;
        // websocket frames are already message boundaries, so we strip CRLF
        if bytes.ends_with(b"\r\n") {
            bytes.truncate(bytes.len() - 2);
        }

        let bytes = bytes.freeze();
        let message = match this.mode {
            Mode::Binary => Message::Binary(bytes),
            Mode::Text => {
                let text = Utf8Bytes::try_from(bytes)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                Message::Text(text)
            }
        };

        this.stream
            .start_send_unpin(message)
            .map_err(websocket_error)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .stream
            .poll_flush_unpin(cx)
            .map_err(websocket_error)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.as_mut()
            .project()
            .stream
            .poll_close_unpin(cx)
            .map_err(websocket_error)
    }
}

fn websocket_uri(scheme: &str, server: &str, port: u16, path: &str) -> String {
    let host = if server.contains(':') && !server.starts_with('[') {
        format!("[{server}]")
    } else {
        server.to_string()
    };
    let path = if path.is_empty() {
        "/".to_string()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    format!("{scheme}://{host}:{port}{path}")
}

fn mode_from_response(response: &http::Response<Option<Vec<u8>>>) -> error::Result<Mode> {
    let protocol = response
        .headers()
        .get(http::header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok());

    match protocol {
        Some(TEXT_SUBPROTOCOL) => Ok(Mode::Text),
        Some(BINARY_SUBPROTOCOL) | None => Ok(Mode::Binary),
        Some(protocol) => Err(error::Error::WebsocketUnsupportedSubprotocol(
            protocol.into(),
        )),
    }
}

fn websocket_error<E>(error: tungstenite::Error) -> E
where
    E: From<io::Error>,
{
    io::Error::new(io::ErrorKind::ConnectionAborted, error).into()
}

impl WebsocketPinger {
    fn new(config: &Config) -> Self {
        let ping_time = Duration::from_secs(config.websocket_ping_time());
        let ping_timeout = Duration::from_secs(u64::from(config.ping_timeout()));

        Self {
            ping_timeout,
            ping_deadline: None,
            ping_interval: time::interval(ping_time),
        }
    }
}

impl Future for WebsocketPinger {
    type Output = Result<PingerMessage, error::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ping_deadline) = self.as_mut().project().ping_deadline.as_pin_mut() {
            match ping_deadline.poll(cx) {
                Poll::Ready(()) => return Poll::Ready(Err(error::Error::PingTimeout)),
                Poll::Pending => (),
            }
        }

        if self
            .as_mut()
            .project()
            .ping_interval
            .poll_tick(cx)
            .is_ready()
        {
            return Poll::Ready(Ok(PingerMessage::Ping));
        }

        Poll::Pending
    }
}
