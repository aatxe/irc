//! Implementation of IRC codec for Tokio.
use bytes::BytesMut;
use tokio_io::codec::{Decoder, Encoder};

use error;
use line::LineCodec;
use message::Message;

/// An IRC codec built around an inner codec.
pub struct IrcCodec {
    inner: LineCodec,
}

impl IrcCodec {
    /// Creates a new instance of IrcCodec wrapping a LineCodec with the specific encoding.
    pub fn new(label: &str) -> error::Result<IrcCodec> {
        LineCodec::new(label).map(|codec| IrcCodec { inner: codec })
    }
}

impl Decoder for IrcCodec {
    type Item = Message;
    type Error = error::ProtocolError;

    fn decode(&mut self, src: &mut BytesMut) -> error::Result<Option<Message>> {
        self.inner.decode(src).and_then(|res| {
            res.map_or(Ok(None), |msg| msg.parse::<Message>().map(Some))
        })
    }
}

impl Encoder for IrcCodec {
    type Item = Message;
    type Error = error::ProtocolError;


    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> error::Result<()> {
        self.inner.encode(msg.to_string(), dst)
    }
}
