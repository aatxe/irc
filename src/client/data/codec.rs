//! A trait that describes a codec that can be used to encode or decode IRC messages.
//! At the moment, this is just a wrapper around [`tokio_util::codec::Decoder`] and [`tokio_util::codec::Encoder`].

use irc_proto::{error::ProtocolError, CapSubCommand, Command, IrcCodec, Response};
use std::fmt::{Debug, Display};

pub use tokio_util::codec::{Decoder, Encoder};

/// A codec that can be used to encode or decode IRC messages.
pub trait MessageCodec:
    Decoder<Item = <Self as MessageCodec>::MsgItem> + Encoder<<Self as MessageCodec>::MsgItem> + Sized
{
    /// The type of the message that this codec expects for messages that we want to send.
    type MsgItem: Display
        + Debug
        + Clone
        + Unpin
        + Sized
        + InternalIrcMessageIncoming
        + InternalIrcMessageOutgoing;

    /// Construct an instance of the codec based on the given character encoding.
    fn try_new(char_encoding: impl AsRef<str>) -> Result<Self, ProtocolError>;
}

#[cfg(feature = "essentials")]
/// Helper trait to feature gate the `From<crate::proto::Command>` dependency.
pub trait InternalIrcMessageOutgoingBase: From<crate::proto::Command> {}

#[cfg(feature = "essentials")]
impl<T> InternalIrcMessageOutgoingBase for T where T: From<crate::proto::Command> {}

#[cfg(not(feature = "essentials"))]
/// Helper trait to feature gate the `From<crate::proto::Command>` dependency.
pub trait InternalIrcMessageOutgoingBase {}

#[cfg(not(feature = "essentials"))]
impl<T> InternalIrcMessageOutgoingBase for T {}

#[cfg(feature = "essentials")]
/// Helper trait to feature gate the `Borrow<crate::proto::Message>` dependency.
pub trait InternalIrcMessageIncomingBase: std::borrow::Borrow<crate::proto::Message> {}

#[cfg(feature = "essentials")]
impl<T> InternalIrcMessageIncomingBase for T where T: std::borrow::Borrow<crate::proto::Message> {}

#[cfg(not(feature = "essentials"))]
/// Helper trait to feature gate the `Borrow<crate::proto::Message>` dependency.
pub trait InternalIrcMessageIncomingBase {}

#[cfg(not(feature = "essentials"))]
impl<T> InternalIrcMessageIncomingBase for T {}

/// An message type that supports decoding the commands necessary to maintain communication with a basic IRC server.
pub trait InternalIrcMessageIncoming: InternalIrcMessageIncomingBase {
    /// Whether or not this message is a `RPL_ENDOFMOTD` response.
    fn is_end_of_motd(&self) -> bool;

    /// Whether or not this message is a `ERR_NOMOTD` response.
    fn is_err_nomotd(&self) -> bool;

    /// Whether or not this message is a `PONG` message.
    fn is_pong(&self) -> bool;

    /// Whether or not this message is a `QUIT` message.
    fn is_quit(&self) -> bool;

    /// If this message is a `PING` message, this returns the payload.
    fn as_ping(&self) -> Option<String>;
}

/// An message type that supports encoding enough message types to maintain communication with an IRC server.
pub trait InternalIrcMessageOutgoing: InternalIrcMessageOutgoingBase {
    // Basic functionality
    // Encode message types that are absolutely necessary for the communication even with basic IRC servers.

    /// Create a `PING` message.
    fn new_ping(server: String) -> Self;

    /// Create a `PONG` message.
    fn new_pong(daemon: String) -> Self;

    /// Create a `CAP END` message (end of capabilities request).
    fn new_cap_end() -> Self;

    /// Create a `NICK` message.
    fn new_nick(nick: String) -> Self;

    /// Create a `USER` message. The user mode will be set to `0`.
    fn new_user(username: String, realname: String) -> Self;

    /// Create a `JOIN` message for channels that do not require authentification.
    fn new_join(channel_list: String) -> Self;

    /// Create a `PART` message.
    fn new_part(channel_list: String) -> Self;

    /// Create a `QUIT` message.
    fn new_quit(message: String) -> Self;

    // User Authentification
    // Encode message types necessary to authenticate users.

    /// Create a `PASS` message.
    fn new_pass(password: String) -> Self;

    /// Create a `JOIN` message for channels that require authentification.
    fn new_authenticated_join(channel_list: String, pass_list: String) -> Self;

    /// Create a `NICKSERV` message.
    fn new_nickserv(commands: Vec<String>) -> Self;
}

/// A message type that supports encoding message types necessary to authenticate a password-protected user.
pub trait OutgoingIrcUserAuthentification {}

impl MessageCodec for IrcCodec {
    type MsgItem = irc_proto::Message;

    fn try_new(char_encoding: impl AsRef<str>) -> Result<Self, ProtocolError> {
        Self::new(char_encoding.as_ref())
    }
}

impl InternalIrcMessageOutgoing for irc_proto::Message {
    fn new_ping(server: String) -> Self {
        Command::PING(server, None).into()
    }

    fn new_pong(daemon: String) -> Self {
        Command::PONG(daemon, None).into()
    }

    fn new_cap_end() -> Self {
        Command::CAP(None, CapSubCommand::END, None, None).into()
    }

    fn new_nick(nick: String) -> Self {
        Command::NICK(nick).into()
    }

    fn new_user(username: String, realname: String) -> Self {
        Command::USER(username, "0".to_owned(), realname).into()
    }

    fn new_join(channel_list: String) -> Self {
        Command::JOIN(channel_list, None, None).into()
    }

    fn new_part(channel_list: String) -> Self {
        Command::PART(channel_list, None).into()
    }

    fn new_quit(message: String) -> Self {
        const DEFAULT_QUIT_MESSAGE: &str = "Powered by Rust.";
        Command::QUIT(Some(if message.is_empty() {
            DEFAULT_QUIT_MESSAGE.to_string()
        } else {
            message
        }))
        .into()
    }

    // User Authentication

    fn new_pass(password: String) -> Self {
        Command::PASS(password).into()
    }

    fn new_authenticated_join(channel_list: String, pass_list: String) -> Self {
        Command::JOIN(channel_list, Some(pass_list), None).into()
    }

    fn new_nickserv(commands: Vec<String>) -> Self {
        Command::NICKSERV(commands).into()
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

    fn is_quit(&self) -> bool {
        matches!(self.command, Command::QUIT(..))
    }

    fn as_ping(&self) -> Option<String> {
        if let Command::PING(ref payload, _) = self.command {
            Some(payload.clone())
        } else {
            None
        }
    }
}
