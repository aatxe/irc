//! Implementation of IRC codec for Tokio.

use std::io;
use proto::line::LineCodec;
use proto::message::Message;
use tokio_core::io::{Codec, EasyBuf};

/// An IRC codec built around an inner codec.
pub struct IrcCodec {
    inner: LineCodec,
}

impl IrcCodec {
    /// Creates a new instance of IrcCodec wrapping a LineCodec with the specifiec encoding.
    pub fn new(label: &str) -> io::Result<IrcCodec> {
        LineCodec::new(label).map(|codec| IrcCodec { inner: codec })
    }
}

impl Codec for IrcCodec {
    type In = Message;
    type Out = Message;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Message>> {
        self.inner.decode(buf).and_then(|res| res.map_or(Ok(None), |msg| {
            msg.parse::<Message>().map(|msg| Some(msg)).map_err(|err| {
                io::Error::new(io::ErrorKind::InvalidInput, err)
            })
        }))
    }

    fn encode(&mut self, msg: Message, buf: &mut Vec<u8>) -> io::Result<()> {
        self.inner.encode(msg.to_string(), buf)
    }
}
