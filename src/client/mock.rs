use std::{
    io::{self, Cursor, Read, Write},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// A fake stream for testing network applications backed by buffers.
#[derive(Clone, Debug)]
pub struct MockStream {
    written: Cursor<Vec<u8>>,
    received: Cursor<Vec<u8>>,
}

impl MockStream {
    /// Creates a new mock stream with nothing to read.
    pub fn empty() -> MockStream {
        MockStream::new(&[])
    }

    /// Creates a new mock stream with the specified bytes to read.
    pub fn new(initial: &[u8]) -> MockStream {
        MockStream {
            written: Cursor::new(vec![]),
            received: Cursor::new(initial.to_owned()),
        }
    }

    /// Gets a slice of bytes representing the data that has been written.
    pub fn written(&self) -> &[u8] {
        self.written.get_ref()
    }

    /// Gets a slice of bytes representing the data that has been received.
    pub fn received(&self) -> &[u8] {
        self.received.get_ref()
    }
}

impl AsyncRead for MockStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let n = self.as_mut().received.read(buf.initialize_unfilled())?;
        buf.advance(n);
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for MockStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Poll::Ready(self.as_mut().written.write(buf))
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(self.as_mut().written.flush())
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }
}
