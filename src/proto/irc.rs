//! Implementation of IRC codec for Tokio.

use std::io;
use bytes::BytesMut;
use tokio_io::codec::{Decoder, Encoder};
use proto::line::LineCodec;
use proto::message::Message;

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

impl Decoder for IrcCodec {
    type Item = Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Message>> {
        self.inner.decode(src).and_then(|res| res.map_or(Ok(None), |msg| {
            msg.parse::<Message>().map(|msg| Some(msg)).map_err(|err| {
                io::Error::new(io::ErrorKind::InvalidInput, err)
            })
        }))
    }
}

impl Encoder for IrcCodec {
    type Item = Message;
    type Error = io::Error;


    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> io::Result<()> {
        self.inner.encode(msg.to_string(), dst)
    }
}
