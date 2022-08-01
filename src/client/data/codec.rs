//! A trait that describes a codec that can be used to encode or decode IRC messages.
//! At the moment, this is just a wrapper around [`tokio_util::codec::Decoder`] and [`tokio_util::codec::Encoder`].

use irc_proto::{error::ProtocolError, Command, IrcCodec, Response};
use std::fmt::{Debug, Display};

pub use tokio_util::codec::{Decoder, Encoder};

/// A codec that can be used to encode or decode IRC messages.
pub trait MessageCodec:
    Decoder<Item = <Self as MessageCodec>::MsgItem> + Encoder<<Self as MessageCodec>::MsgItem> + Sized
{
    /// The type of the message that this codec expects for messages that we want to send.
    type MsgItem: Display + Debug + Clone + InternalIrcMessageIncoming + InternalIrcMessageOutgoing;

    /// Construct an instance of the codec based on the given character encoding.
    fn try_new(char_encoding: impl AsRef<str>) -> Result<Self, ProtocolError>;
}

/// An message type that supports decoding all commands necessary to maintain communication with an IRC server.
/// For this crate, such messages have to support the `RESPONSE` and `PING` commands.
pub trait InternalIrcMessageIncoming {
    /// Whether or not this message is a `RPL_ENDOFMOTD` response.
    fn is_end_of_motd(&self) -> bool;

    /// Whether or not this message is a `ERR_NOMOTD` response.
    fn is_err_nomotd(&self) -> bool;

    /// Whether or not this message is a `PONG` message.
    fn is_pong(&self) -> bool;

    /// If this message is a `PING` message, this returns the payload.
    fn as_ping(&self) -> Option<String>;
}

/// An message type that supports encoding all commands necessary to maintain communication with an IRC server.
/// For this crate, such messages have to support the `PING` and `PONG` commands.
pub trait InternalIrcMessageOutgoing {
    /// Create a `PING` message.
    fn new_ping(server: String, server_fwd: Option<String>) -> Self;

    /// Create a `PONG` message.
    fn new_pong(daemon: String, daemon_fwd: Option<String>) -> Self;
}

impl MessageCodec for IrcCodec {
    type MsgItem = irc_proto::Message;

    fn try_new(char_encoding: impl AsRef<str>) -> Result<Self, ProtocolError> {
        Self::new(char_encoding.as_ref())
    }
}

impl InternalIrcMessageOutgoing for irc_proto::Message {
    fn new_ping(server: String, server_fwd: Option<String>) -> Self {
        Command::PING(server, server_fwd).into()
    }

    fn new_pong(daemon: String, daemon_fwd: Option<String>) -> Self {
        Command::PONG(daemon, daemon_fwd).into()
    }
}

impl InternalIrcMessageIncoming for irc_proto::Message {
    fn is_end_of_motd(&self) -> bool {
        matches!(self.command, Command::Response(Response::RPL_ENDOFMOTD, _))
    }
    fn is_err_nomotd(&self) -> bool {
        matches!(self.command, Command::Response(Response::ERR_NOMOTD, _))
    }

    fn is_pong(&self) -> bool {
        matches!(self.command, Command::PONG(..))
    }

    fn as_ping(&self) -> Option<String> {
        if let Command::PING(ref payload, _) = self.command {
            Some(payload.clone())
        } else {
            None
        }
    }
}
