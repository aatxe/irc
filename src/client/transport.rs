//! An IRC transport that wraps an IRC-framed stream to provide a number of features including
//! automatic PING replies, automatic sending of PINGs, and message rate-limiting. This can be used
//! as the basis for implementing a more full IRC client.
use std::collections::VecDeque;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::{Duration, Instant};

use futures::{Async, AsyncSink, Future, Poll, Sink, StartSend, Stream};
use chrono::prelude::*;
use tokio_codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_timer;
use tokio_timer::{Interval, Sleep, Timer};

use error;
use client::data::Config;
use proto::{Command, IrcCodec, Message};

/// An IRC transport that handles core functionality for the IRC protocol. This is used in the
/// implementation of `Connection` and ultimately `IrcServer`, and plays an important role in
/// handling connection timeouts, message throttling, and ping response.
pub struct IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    /// The inner connection framed with an `IrcCodec`.
    inner: Framed<T, IrcCodec>,
    /// A timer used in computing windows for message throttling.
    burst_timer: Timer,
    /// A queue of tasks used to implement message throttling.
    rolling_burst_window: VecDeque<Sleep>,
    /// The amount of time that each window for throttling should last (in seconds).
    burst_window_length: u64,
    /// The maximum number of messages that can be sent in each window.
    max_burst_messages: u64,
    /// The number of messages sent in the current window.
    current_burst_messages: u64,
    /// A timer used to determine when to send the next ping messages to the server.
    ping_timer: Interval,
    /// The amount of time to wait before timing out from no ping response.
    ping_timeout: u64,
    /// The last data sent with a ping.
    last_ping_data: String,
    /// The instant that the last ping was sent to the server.
    last_ping_sent: Instant,
    /// The instant that the last pong was received from the server.
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
            inner,
            burst_timer: tokio_timer::wheel().build(),
            rolling_burst_window: VecDeque::new(),
            burst_window_length: u64::from(config.burst_window_length()),
            max_burst_messages: u64::from(config.max_messages_in_burst()),
            current_burst_messages: 0,
            ping_timer: timer.interval(Duration::from_secs(u64::from(config.ping_time()))),
            ping_timeout: u64::from(config.ping_timeout()),
            last_ping_data: String::new(),
            last_ping_sent: Instant::now(),
            last_pong_received: Instant::now(),
        }
    }

    /// Gets the inner stream underlying the `IrcTransport`.
    pub fn into_inner(self) -> Framed<T, IrcCodec> {
        self.inner
    }

    /// Determines whether or not the transport has hit the ping timeout.
    fn ping_timed_out(&self) -> bool {
        if self.last_pong_received < self.last_ping_sent {
            self.last_ping_sent.elapsed().as_secs() >= self.ping_timeout
        } else {
            false
        }
    }

    /// Sends a ping via the transport.
    fn send_ping(&mut self) -> error::Result<()> {
        // Creates new ping data using the local timestamp.
        let last_ping_data = format!("{}", Local::now().timestamp());
        let data = last_ping_data.clone();
        let result = self.start_send(Command::PING(data, None).into())?;
        if let AsyncSink::Ready = result {
            self.poll_complete()?;
            // If we succeeded in sending the ping, we will update when the last ping was sent, and
            // the data that was sent with it.
            self.last_ping_sent = Instant::now();
            self.last_ping_data = last_ping_data;
        }
        Ok(())
    }

    /// Polls the most recent burst window from the queue, always returning `NotReady` if none are
    /// left for whatever reason.
    fn rolling_burst_window_front(&mut self) -> Result<Async<()>, tokio_timer::TimerError> {
        self.rolling_burst_window.front_mut().map(|w| w.poll()).unwrap_or(Ok(Async::NotReady))
    }
}

impl<T> Stream for IrcTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Item = Message;
    type Error = error::IrcError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // If the ping timeout has been reached, we close the connection out and return an error.
        if self.ping_timed_out() {
            self.close()?;
            return Err(error::IrcError::PingTimeout)
        }

        // We poll both streams before doing any work because it is important to ensure that the
        // task is correctly woken up when they are ready.
        let timer_poll = self.ping_timer.poll()?;
        let inner_poll = self.inner.poll()?;

        match (inner_poll, timer_poll) {
            // If neither the stream nor the ping timer are ready, the transport is not ready.
            (Async::NotReady, Async::NotReady) => Ok(Async::NotReady),

            // If there's nothing available yet from the stream, but the ping timer is ready, we
            // simply send a ping and indicate that the transport has nothing to yield yet.
            (Async::NotReady, Async::Ready(msg)) => {
                assert!(msg.is_some());
                self.send_ping()?;
                Ok(Async::NotReady)
            }

            // If the stream yields `None`, the connection has been terminated. Thus, we don't need
            // to worry about checking the ping timer, and can instead indicate that the transport
            // has been terminated.
            (Async::Ready(None), _) => Ok(Async::Ready(None)),

            // If we have a new message available from the stream, we'll need to do some work, and
            // then yield the message.
            (Async::Ready(Some(msg)), _) => {
                // If the ping timer has returned, it is time to send another `PING` message!
                if let Async::Ready(msg) = timer_poll {
                    assert!(msg.is_some());
                    self.send_ping()?;
                }

                match msg.command {
                    // On receiving a `PING` message from the server, we automatically respond with
                    // the appropriate `PONG` message to keep the connection alive for transport.
                    Command::PING(ref data, _) => {
                        let result = self.start_send(Command::PONG(data.to_owned(), None).into())?;
                        assert!(result.is_ready());
                        self.poll_complete()?;
                    }

                    // Check `PONG` responses from the server. If it matches, we will update the
                    // last instant that the pong was received. This will prevent timeout.
                    Command::PONG(ref data, None) |
                    Command::PONG(_, Some(ref data)) => {
                        if self.last_ping_data == data[..] {
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
    type SinkError = error::IrcError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        // If the ping timeout has been reached, we close the connection out and return an error.
        if self.ping_timed_out() {
            self.close()?;
            return Err(error::IrcError::PingTimeout)
        }

        // Check if the oldest message in the rolling window is discounted.
        if let Async::Ready(()) = self.rolling_burst_window_front()? {
            self.current_burst_messages -= 1;
            self.rolling_burst_window.pop_front();
        }

        // Throttling if too many messages have been sent recently.
        if self.current_burst_messages >= self.max_burst_messages {
            // When throttled, we know we need to finish sending what's already queued up.
            self.poll_complete()?;
            return Ok(AsyncSink::NotReady(item))
        }

        match self.inner.start_send(item)? {
            AsyncSink::NotReady(item) => Ok(AsyncSink::NotReady(item)),
            AsyncSink::Ready => {
                self.current_burst_messages += 1;
                self.rolling_burst_window.push_back(self.burst_timer.sleep(Duration::from_secs(
                    self.burst_window_length
                )));
                Ok(AsyncSink::Ready)
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        // If the ping timeout has been reached, we close the connection out and return an error.
        if self.ping_timed_out() {
            self.close()?;
            return Err(error::IrcError::PingTimeout)
        }

        // If it's time to send a ping, we should do it! This is necessary to ensure that the
        // sink half will close even if the stream half closed without a ping timeout.
        if let Async::Ready(msg) = self.ping_timer.poll()? {
            assert!(msg.is_some());
            self.send_ping()?;
        }

        Ok(self.inner.poll_complete()?)
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        self.inner.close().map_err(|e| e.into())
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
        self.sent.read().map_err(|_| error::IrcError::PoisonedLog)
    }

    /// Gets a read guard for all the messages received on the transport.
    pub fn received(&self) -> error::Result<RwLockReadGuard<Vec<Message>>> {
        self.received.read().map_err(|_| error::IrcError::PoisonedLog)
    }
}

/// A logged version of the `IrcTransport` that records all sent and received messages.
/// Note: this will introduce some performance overhead by cloning all messages.
pub struct Logged<T> where T: AsyncRead + AsyncWrite {
    inner: IrcTransport<T>,
    view: LogView,
}

impl<T> Logged<T> where T: AsyncRead + AsyncWrite {
    /// Wraps the given `IrcTransport` in logging.
    pub fn wrap(inner: IrcTransport<T>) -> Logged<T> {
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
    T: AsyncRead + AsyncWrite,
{
    type Item = Message;
    type Error = error::IrcError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match try_ready!(self.inner.poll()) {
            Some(msg) => {
                let recv: error::Result<_> = self.view.received.write().map_err(|_| {
                    error::IrcError::PoisonedLog
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
    type SinkError = error::IrcError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        let res = self.inner.start_send(item.clone())?;
        let sent: error::Result<_> = self.view.sent.write().map_err(|_| {
            error::IrcError::PoisonedLog
        });
        sent?.push(item);
        Ok(res)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(self.inner.poll_complete()?)
    }
}
