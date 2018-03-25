//! Implementation of IRC codec for Tokio.
use std::marker::PhantomData;

use bytes::BytesMut;
use tokio_io::codec::{Decoder, Encoder};

use error;
use line::LineCodec;
use message::{Message, OwnedMessage};

/// An IRC codec built around an inner codec.
pub struct IrcCodec<'a> {
    inner: LineCodec,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> IrcCodec<'a> {
    /// Creates a new instance of IrcCodec wrapping a LineCodec with the specific encoding.
    pub fn new(label: &str) -> error::Result<IrcCodec<'a>> {
        LineCodec::new(label).map(|codec| IrcCodec {
            inner: codec,
            _lifetime: PhantomData::default()
        })
    }
}

impl<'a> Decoder for IrcCodec<'a> {
    type Item = OwnedMessage;
    type Error = error::ProtocolError;

    fn decode(&mut self, src: &mut BytesMut) -> error::Result<Option<OwnedMessage>> {
        self.inner.decode(src).and_then(|res| {
            res.map_or(Ok(None), |msg| msg.parse::<OwnedMessage>().map(Some))
        })
    }
}

impl<'a> Encoder for IrcCodec<'a> {
    type Item = Message<'a>;
    type Error = error::ProtocolError;


    fn encode(&mut self, msg: Self::Item, dst: &mut BytesMut) -> error::Result<()> {
        self.inner.encode(msg.to_string(), dst)
    }
}
