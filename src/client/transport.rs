//! An IRC transport that wraps an IRC-framed stream to provide a number of features including
//! automatic PING replies, automatic sending of PINGs, and message rate-limiting.
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::{Duration, Instant};

use futures::{Async, Poll, Sink, StartSend, Stream};
use chrono::prelude::*;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::Framed;
use tokio_timer;
use tokio_timer::Interval;

use error;
use client::data::Config;
use proto::{Command, IrcCodec, Message};

/// An IRC transport that handles core functionality.
pub struct IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    inner: Framed<T, IrcCodec>,
    burst_timer: Interval,
    max_burst_messages: u32,
    current_burst_messages: u32,
    ping_timer: Interval,
    ping_timeout: u64,
    last_ping_data: String,
    last_ping_sent: Instant,
    last_pong_received: Instant,
}

impl<T> IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    /// Creates a new `IrcTransport` from the given IRC stream.
    pub fn new(config: &Config, inner: Framed<T, IrcCodec>) -> IrcTransport<T> {
        let timer = tokio_timer::wheel().build();
        IrcTransport {
            inner: inner,
            burst_timer: timer.interval(Duration::from_secs(config.burst_window_length() as u64)),
            max_burst_messages: config.max_messages_in_burst(),
            current_burst_messages: 0,
            ping_timer: timer.interval(Duration::from_secs(config.ping_time() as u64)),
            ping_timeout: config.ping_timeout() as u64,
            last_ping_data: String::new(),
            last_ping_sent: Instant::now(),
            last_pong_received: Instant::now(),
        }
    }

    /// Gets the inner stream underlying the `IrcTransport`.
    pub fn into_inner(self) -> Framed<T, IrcCodec> {
        self.inner
    }

    fn ping_timed_out(&self) -> bool {
        if self.last_pong_received < self.last_ping_sent {
            self.last_ping_sent.elapsed().as_secs() >= self.ping_timeout
        } else {
            false
        }
    }

    fn send_ping(&mut self) -> error::Result<()> {
        self.last_ping_sent = Instant::now();
        self.last_ping_data = format!("{}", Local::now().timestamp());
        let data = self.last_ping_data.clone();
        let result = self.start_send(Command::PING(data, None).into())?;
        assert!(result.is_ready());
        self.poll_complete()?;
        Ok(())
    }
}

impl<T> Stream for IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = Message;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.ping_timed_out() {
            self.close()?;
            return Err(error::ErrorKind::PingTimeout.into())
        }

        let timer_poll = self.ping_timer.poll()?;
        let inner_poll = self.inner.poll()?;

        match (inner_poll, timer_poll) {
            (Async::NotReady, Async::NotReady) => Ok(Async::NotReady),
            (Async::NotReady, Async::Ready(msg)) => {
                assert!(msg.is_some());
                self.send_ping()?;
                Ok(Async::NotReady)
            }
            (Async::Ready(None), _) => Ok(Async::Ready(None)),
            (Async::Ready(Some(msg)), _) => {
                match timer_poll {
                    Async::Ready(msg) => {
                        assert!(msg.is_some());
                        self.send_ping()?;
                    }
                    Async::NotReady => (),
                }

                match msg.command {
                    // Automatically respond to PINGs from the server.
                    Command::PING(ref data, _) => {
                        let result = self.start_send(Command::PONG(data.to_owned(), None).into())?;
                        assert!(result.is_ready());
                        self.poll_complete()?;
                    }
                    // Check PONG responses from the server.
                    Command::PONG(ref data, None) |
                    Command::PONG(_, Some(ref data)) => {
                        if self.last_ping_data == &data[..] {
                            self.last_pong_received = Instant::now();
                        }
                    }
                    _ => (),
                }

                Ok(Async::Ready(Some(msg)))
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
        if self.ping_timed_out() {
            self.close()?;
            Err(error::ErrorKind::PingTimeout.into())
        } else {
            Ok(self.inner.start_send(item)?)
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        if self.ping_timed_out() {
            self.close()?;
            Err(error::ErrorKind::PingTimeout.into())
        } else {
            match self.burst_timer.poll()? {
                Async::NotReady => (),
                Async::Ready(msg) => {
                    assert!(msg.is_some());
                    self.current_burst_messages = 0;
                }
            }

            // Throttling if too many messages have been sent recently.
            if self.current_burst_messages >= self.max_burst_messages {
                return Ok(Async::NotReady)
            }

            match self.inner.poll_complete()? {
                Async::NotReady => Ok(Async::NotReady),
                Async::Ready(()) => {
                    self.current_burst_messages += 1;
                    Ok(Async::Ready(()))
                }
            }
        }
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.close()
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
