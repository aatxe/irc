//! Implementation of IRC codec for Tokio.

use std::io;
use proto::line::LineCodec;
use proto::message::Message;
use tokio_core::io::{Codec, EasyBuf};

/// An IRC codec built around an inner codec.
pub struct IrcCodec<C: Codec> {
    inner: C,
}

impl IrcCodec<LineCodec> {
    /// Creates a new instance of IrcCodec wrapping a LineCodec with the specifiec encoding.
    pub fn new(label: &str) -> io::Result<IrcCodec<LineCodec>> {
        LineCodec::new(label).map(|codec| IrcCodec::from_codec(codec))
    }
}

impl<C> IrcCodec<C> where C: Codec<In = String, Out = String> {
    /// Creates a new instance of IrcCodec from the specified inner codec.
    pub fn from_codec(codec: C) -> IrcCodec<C> {
        IrcCodec { inner: codec }
    }
}

impl<C> Codec for IrcCodec<C> where C: Codec<In = String, Out = String> {
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
