//! Implementation of IRC protocol for Tokio.

use std::io;
use proto::line::LineCodec;
use proto::message::Message;
use tokio_core::io::{Codec, EasyBuf, Framed, Io};
use tokio_proto::pipeline::{ServerProto, ClientProto};

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

/// Implementation of the IRC protocol backed by a line-delimited codec.
pub struct IrcProto {
    encoding_label: String,
}

impl IrcProto {
    /// Creates a new IrcProto using the specified WHATWG encoding label.
    fn new(label: &str) -> IrcProto {
        IrcProto { encoding_label: label.to_owned() }
    }
}

impl<T> ClientProto<T> for IrcProto where T: Io + 'static {
    type Request = Message;
    type Response = Message;

    type Transport = Framed<T, IrcCodec<LineCodec>>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(try!(IrcCodec::new(&self.encoding_label))))
    }
}

impl<T> ServerProto<T> for IrcProto where T: Io + 'static {
    type Request = Message;
    type Response = Message;

    type Transport = Framed<T, IrcCodec<LineCodec>>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(try!(IrcCodec::new(&self.encoding_label))))
    }
}
