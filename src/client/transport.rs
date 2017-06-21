//! An IRC transport that wraps an IRC-framed stream to provide automatic PING replies.
use std::io;
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
