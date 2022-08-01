//! An IRC transport that wraps an IRC-framed stream to provide a number of features including
//! automatic PING replies, automatic sending of PINGs, and message rate-limiting. This can be used
//! as the basis for implementing a more full IRC client.
use std::{
    pin::Pin,
    sync::{Arc, RwLock, RwLockReadGuard},
    task::{Context, Poll},
    time::Duration,
};

use chrono::prelude::*;
use futures_util::{future::Future, ready, sink::Sink, stream::Stream};
use pin_project::pin_project;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    time::{self, Interval, Sleep},
};
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::{client::data::Config, error};

use super::data::codec::{InternalIrcMessageIncoming, InternalIrcMessageOutgoing, MessageCodec};

/// Pinger-based futures helper.
#[pin_project]
struct Pinger<Msg> {
    tx: UnboundedSender<Msg>,
    // Whether this pinger pings.
    enabled: bool,
    /// The amount of time to wait before timing out from no ping response.
    ping_timeout: Duration,
    /// The instant that the last ping was sent to the server.
    #[pin]
    ping_deadline: Option<Sleep>,
    /// The interval at which to send pings.
    #[pin]
    ping_interval: Interval,
}

impl<Msg> Pinger<Msg>
where
    Msg: InternalIrcMessageOutgoing + InternalIrcMessageIncoming,
{
    /// Construct a new pinger helper.
    pub fn new(tx: UnboundedSender<Msg>, config: &Config) -> Pinger<Msg> {
        let ping_time = Duration::from_secs(u64::from(config.ping_time()));
        let ping_timeout = Duration::from_secs(u64::from(config.ping_timeout()));

        Self {
            tx,
            enabled: false,
            ping_timeout,
            ping_deadline: None,
            ping_interval: time::interval(ping_time),
        }
    }

    /// Handle an incoming message.
    fn handle_message(self: Pin<&mut Self>, message: &Msg) -> error::Result<()> {
        if message.is_end_of_motd() || message.is_err_nomotd() {
            *self.project().enabled = true;
        } else if let Some(ping_payload) = message.as_ping() {
            // On receiving a `PING` message from the server, we automatically respond with
            // the appropriate `PONG` message to keep the connection alive for transport.
            self.send_pong(&ping_payload)?;
        } else if message.is_pong() {
            // Check `PONG` responses from the server. If it matches, we will update the
            // last instant that the pong was received. This will prevent timeout.
            log::trace!("Received PONG");
            self.project().ping_deadline.set(None);
        }

        Ok(())
    }

    /// Send a pong.
    fn send_pong(self: Pin<&mut Self>, data: &str) -> error::Result<()> {
        self.project()
            .tx
            .send(Msg::new_pong(data.to_owned(), None))?;
        Ok(())
    }

    /// Sends a ping via the transport.
    fn send_ping(self: Pin<&mut Self>) -> error::Result<()> {
        log::trace!("Sending PING");

        // Creates new ping data using the local timestamp.
        let data = format!("{}", Local::now().timestamp());

        let mut this = self.project();

        this.tx.send(Msg::new_ping(data, None))?;

        if this.ping_deadline.is_none() {
            let ping_deadline = time::sleep(*this.ping_timeout);
            this.ping_deadline.set(Some(ping_deadline));
        }

        Ok(())
    }
}

impl<Msg> Future for Pinger<Msg>
where
    Msg: InternalIrcMessageOutgoing + InternalIrcMessageIncoming,
{
    type Output = Result<(), error::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ping_deadline) = self.as_mut().project().ping_deadline.as_pin_mut() {
            match ping_deadline.poll(cx) {
                Poll::Ready(()) => return Poll::Ready(Err(error::Error::PingTimeout)),
                Poll::Pending => (),
            }
        }

        if let Poll::Ready(_) = self.as_mut().project().ping_interval.poll_tick(cx) {
            if *self.as_mut().project().enabled {
                self.as_mut().send_ping()?;
            }
        }

        Poll::Pending
    }
}

/// An IRC transport that handles core functionality for the IRC protocol. This is used in the
/// implementation of `Connection` and ultimately `IrcServer`, and plays an important role in
/// handling connection timeouts, message throttling, and ping response.
#[pin_project]
pub struct Transport<T, Codec>
where
    Codec: MessageCodec,
{
    /// The inner connection framed with a Codec that implements [`MessageCodec`]. By default, this is [`IrcCodec`].
    #[pin]
    inner: Framed<T, Codec>,
    /// Helper for handle pinging.
    #[pin]
    pinger: Option<Pinger<Codec::MsgItem>>,
}

impl<T, Codec> Transport<T, Codec>
where
    T: Unpin + AsyncRead + AsyncWrite,
    Codec: MessageCodec,
{
    /// Creates a new `Transport` from the given IRC stream.
    pub fn new(
        config: &Config,
        inner: Framed<T, Codec>,
        tx: UnboundedSender<Codec::MsgItem>,
    ) -> Transport<T, Codec> {
        let pinger = Some(Pinger::new(tx, config));

        Transport { inner, pinger }
    }

    /// Gets the inner stream underlying the `Transport`.
    pub fn into_inner(self) -> Framed<T, Codec> {
        self.inner
    }
}

impl<T, Codec> Stream for Transport<T, Codec>
where
    T: Unpin + AsyncRead + AsyncWrite,
    Codec: MessageCodec,
    error::Error: From<<Codec as Decoder>::Error>,
{
    type Item = Result<Codec::MsgItem, error::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<<Self as Stream>::Item>> {
        if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
            match pinger.poll(cx) {
                Poll::Ready(result) => result?,
                Poll::Pending => (),
            }
        }

        let result = ready!(self.as_mut().project().inner.poll_next(cx));

        let message = match result {
            None => return Poll::Ready(None),
            Some(message) => message?,
        };

        if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
            pinger.handle_message(&message)?;
        }

        Poll::Ready(Some(Ok(message)))
    }
}

impl<T, Codec> Sink<Codec::MsgItem> for Transport<T, Codec>
where
    T: Unpin + AsyncRead + AsyncWrite,
    Codec: MessageCodec,
    error::Error: From<<Codec as Encoder<Codec::MsgItem>>::Error>,
{
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.project().inner.poll_ready(cx))?;
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Codec::MsgItem) -> Result<(), Self::Error> {
        log::trace!("[SEND] {}", item);
        self.project().inner.start_send(item)?;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.project().inner.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.project().inner.poll_close(cx))?;
        Poll::Ready(Ok(()))
    }
}

/// A view of the logs from a particular `Logged` transport.
#[derive(Clone, Debug)]
pub struct LogView<Msg> {
    sent: Arc<RwLock<Vec<Msg>>>,
    received: Arc<RwLock<Vec<Msg>>>,
}

impl<Msg> LogView<Msg> {
    /// Gets a read guard for all the messages sent on the transport.
    pub fn sent(&self) -> error::Result<RwLockReadGuard<Vec<Msg>>> {
        self.sent.read().map_err(|_| error::Error::PoisonedLog)
    }

    /// Gets a read guard for all the messages received on the transport.
    pub fn received(&self) -> error::Result<RwLockReadGuard<Vec<Msg>>> {
        self.received.read().map_err(|_| error::Error::PoisonedLog)
    }
}

/// A logged version of the `Transport` that records all sent and received messages.
/// Note: this will introduce some performance overhead by cloning all messages.
#[pin_project]
pub struct Logged<T, Codec>
where
    Codec: MessageCodec,
{
    #[pin]
    inner: Transport<T, Codec>,
    view: LogView<Codec::MsgItem>,
}

impl<T, Codec> Logged<T, Codec>
where
    T: AsyncRead + AsyncWrite,
    Codec: MessageCodec,
{
    /// Wraps the given `Transport` in logging.
    pub fn wrap(inner: Transport<T, Codec>) -> Logged<T, Codec> {
        Logged {
            inner,
            view: LogView {
                sent: Arc::new(RwLock::new(vec![])),
                received: Arc::new(RwLock::new(vec![])),
            },
        }
    }

    /// Gets a view of the logging for this transport.
    pub fn view(&self) -> LogView<Codec::MsgItem> {
        self.view.clone()
    }
}

impl<T, Codec> Stream for Logged<T, Codec>
where
    T: Unpin + AsyncRead + AsyncWrite,
    Codec: MessageCodec,
    error::Error: From<<Codec as Decoder>::Error>,
{
    type Item = Result<Codec::MsgItem, error::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match ready!(this.inner.poll_next(cx)) {
            Some(msg) => {
                let msg = msg?;

                this.view
                    .received
                    .write()
                    .map_err(|_| error::Error::PoisonedLog)?
                    .push(msg.clone());

                Poll::Ready(Some(Ok(msg)))
            }
            None => Poll::Ready(None),
        }
    }
}

impl<T, Codec> Sink<Codec::MsgItem> for Logged<T, Codec>
where
    T: Unpin + AsyncRead + AsyncWrite,
    Codec: MessageCodec,
    error::Error: From<<Codec as Encoder<Codec::MsgItem>>::Error>,
{
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Codec::MsgItem) -> Result<(), Self::Error> {
        let this = self.project();

        this.inner.start_send(item.clone())?;

        this.view
            .sent
            .write()
            .map_err(|_| error::Error::PoisonedLog)?
            .push(item);

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }
}
