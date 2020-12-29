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
use tokio_util::codec::Framed;

use crate::{
    client::data::Config,
    error,
    proto::{Command, IrcCodec, Message, Response},
};

/// Pinger-based futures helper.
#[pin_project]
struct Pinger {
    tx: UnboundedSender<Message>,
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

impl Pinger {
    /// Construct a new pinger helper.
    pub fn new(tx: UnboundedSender<Message>, config: &Config) -> Pinger {
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
    fn handle_message(self: Pin<&mut Self>, message: &Message) -> error::Result<()> {
        match message.command {
            Command::Response(Response::RPL_ENDOFMOTD, _)
            | Command::Response(Response::ERR_NOMOTD, _) => {
                *self.project().enabled = true;
            }
            // On receiving a `PING` message from the server, we automatically respond with
            // the appropriate `PONG` message to keep the connection alive for transport.
            Command::PING(ref data, _) => {
                self.send_pong(data)?;
            }
            // Check `PONG` responses from the server. If it matches, we will update the
            // last instant that the pong was received. This will prevent timeout.
            Command::PONG(_, None) | Command::PONG(_, Some(_)) => {
                log::trace!("Received PONG");
                self.project().ping_deadline.set(None);
            }
            _ => (),
        }

        Ok(())
    }

    /// Send a pong.
    fn send_pong(self: Pin<&mut Self>, data: &str) -> error::Result<()> {
        self.project()
            .tx
            .send(Command::PONG(data.to_owned(), None).into())?;
        Ok(())
    }

    /// Sends a ping via the transport.
    fn send_ping(self: Pin<&mut Self>) -> error::Result<()> {
        log::trace!("Sending PING");

        // Creates new ping data using the local timestamp.
        let data = format!("{}", Local::now().timestamp());

        let mut this = self.project();

        this.tx.send(Command::PING(data.clone(), None).into())?;

        if this.ping_deadline.is_none() {
            let ping_deadline = time::sleep(*this.ping_timeout);
            this.ping_deadline.set(Some(ping_deadline));
        }

        Ok(())
    }
}

impl Future for Pinger {
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
pub struct Transport<T> {
    /// The inner connection framed with an `IrcCodec`.
    #[pin]
    inner: Framed<T, IrcCodec>,
    /// Helper for handle pinging.
    #[pin]
    pinger: Option<Pinger>,
}

impl<T> Transport<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    /// Creates a new `Transport` from the given IRC stream.
    pub fn new(
        config: &Config,
        inner: Framed<T, IrcCodec>,
        tx: UnboundedSender<Message>,
    ) -> Transport<T> {
        let pinger = Some(Pinger::new(tx, config));

        Transport { inner, pinger }
    }

    /// Gets the inner stream underlying the `Transport`.
    pub fn into_inner(self) -> Framed<T, IrcCodec> {
        self.inner
    }
}

impl<T> Stream for Transport<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    type Item = Result<Message, error::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
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

impl<T> Sink<Message> for Transport<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.project().inner.poll_ready(cx))?;
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
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
pub struct LogView {
    sent: Arc<RwLock<Vec<Message>>>,
    received: Arc<RwLock<Vec<Message>>>,
}

impl LogView {
    /// Gets a read guard for all the messages sent on the transport.
    pub fn sent(&self) -> error::Result<RwLockReadGuard<Vec<Message>>> {
        self.sent.read().map_err(|_| error::Error::PoisonedLog)
    }

    /// Gets a read guard for all the messages received on the transport.
    pub fn received(&self) -> error::Result<RwLockReadGuard<Vec<Message>>> {
        self.received.read().map_err(|_| error::Error::PoisonedLog)
    }
}

/// A logged version of the `Transport` that records all sent and received messages.
/// Note: this will introduce some performance overhead by cloning all messages.
#[pin_project]
pub struct Logged<T> {
    #[pin]
    inner: Transport<T>,
    view: LogView,
}

impl<T> Logged<T>
where
    T: AsyncRead + AsyncWrite,
{
    /// Wraps the given `Transport` in logging.
    pub fn wrap(inner: Transport<T>) -> Logged<T> {
        Logged {
            inner,
            view: LogView {
                sent: Arc::new(RwLock::new(vec![])),
                received: Arc::new(RwLock::new(vec![])),
            },
        }
    }

    /// Gets a view of the logging for this transport.
    pub fn view(&self) -> LogView {
        self.view.clone()
    }
}

impl<T> Stream for Logged<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    type Item = Result<Message, error::Error>;

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

impl<T> Sink<Message> for Logged<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    type Error = error::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
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
