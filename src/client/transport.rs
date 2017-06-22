//! An IRC transport that wraps an IRC-framed stream to provide automatic PING replies.
use std::io;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::Instant;
use error;
use client::data::Config;
use proto::{Command, IrcCodec, Message};
use futures::{Async, Poll, Sink, StartSend, Stream};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;

/// An IRC transport that handles automatically replying to PINGs.
pub struct IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    inner: Framed<T, IrcCodec>,
    ping_timeout: u64,
    last_ping: Instant,
}

impl<T> IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    /// Creates a new `IrcTransport` from the given IRC stream.
    pub fn new(config: &Config, inner: Framed<T, IrcCodec>) -> IrcTransport<T> {
        IrcTransport {
            inner: inner,
            ping_timeout: config.ping_time() as u64,
            last_ping: Instant::now(),
        }
    }

    /// Gets the inner stream underlying the `IrcTransport`.
    pub fn into_inner(self) -> Framed<T, IrcCodec> {
        self.inner
    }
}

impl<T> Stream for IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = Message;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.last_ping.elapsed().as_secs() >= self.ping_timeout {
            self.close()?;
            Err(
                io::Error::new(io::ErrorKind::ConnectionReset, "Ping timed out.").into(),
            )
        } else {
            loop {
                match try_ready!(self.inner.poll()) {
                    Some(Message { command: Command::PING(ref data, _), .. }) => {
                        self.last_ping = Instant::now();
                        let result = self.start_send(Command::PONG(data.to_owned(), None).into())?;
                        assert!(result.is_ready());
                        self.poll_complete()?;
                    }
                    message => return Ok(Async::Ready(message)),
                }
            }
        }
    }
}

impl<T> Sink for IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type SinkItem = Message;
    type SinkError = error::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        Ok(self.inner.start_send(item)?)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.inner.poll_complete()?)
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
        self.sent.read().map_err(
            |_| error::ErrorKind::PoisonedLog.into(),
        )
    }

    /// Gets a read guard for all the messages received on the transport.
    pub fn received(&self) -> error::Result<RwLockReadGuard<Vec<Message>>> {
        self.received.read().map_err(
            |_| error::ErrorKind::PoisonedLog.into(),
        )
    }
}

/// A logged version of the `IrcTransport` that records all sent and received messages.
/// Note: this will introduce some performance overhead by cloning all messages.
pub struct Logged<T>
where
    T: AsyncRead + AsyncWrite,
{
    inner: IrcTransport<T>,
    view: LogView,
}

impl<T> Logged<T>
where
    T: AsyncRead + AsyncWrite,
{
    /// Wraps the given `IrcTransport` in logging.
    pub fn wrap(inner: IrcTransport<T>) -> Logged<T> {
        Logged {
            inner: inner,
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
    T: AsyncRead + AsyncWrite,
{
    type Item = Message;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match try_ready!(self.inner.poll()) {
            Some(msg) => {
                let recv: error::Result<_> = self.view.received.write().map_err(|_| {
                    error::ErrorKind::PoisonedLog.into()
                });
                recv?.push(msg.clone());
                Ok(Async::Ready(Some(msg)))
            }
            None => Ok(Async::Ready(None)),
        }
    }
}

impl<T> Sink for Logged<T>
where
    T: AsyncRead + AsyncWrite,
{
    type SinkItem = Message;
    type SinkError = error::Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let res = self.inner.start_send(item.clone())?;
        let sent: error::Result<_> = self.view.sent.write().map_err(|_| {
            error::ErrorKind::PoisonedLog.into()
        });
        sent?.push(item);
        Ok(res)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.inner.poll_complete()?)
    }
}
