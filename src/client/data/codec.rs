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

    /// Sanitizes the input string by cutting up to (and including) the first occurence of a line
    /// terminiating phrase (`\r\n`, `\r`, or `\n`). This is used in sending messages through the
    /// codec to prevent the injection of additional commands.
    fn sanitize(mut data: String) -> String {
        // n.b. ordering matters here to prefer "\r\n" over "\r"
        if let Some((pos, len)) = ["\r\n", "\r", "\n"]
            .iter()
            .flat_map(|needle| data.find(needle).map(|pos| (pos, needle.len())))
            .min_by_key(|&(pos, _)| pos)
        {
            data.truncate(pos + len);
        }
        data
    }
}

#[cfg(feature = "essentials")]
/// Helper trait to feature gate the [`From<crate::proto::Command>`] dependency (which is currently required anything other than barebones mode).
pub trait InternalIrcMessageOutgoingBase: From<crate::proto::Command> {}

#[cfg(feature = "essentials")]
impl<T> InternalIrcMessageOutgoingBase for T where T: From<crate::proto::Command> {}

#[cfg(not(feature = "essentials"))]
/// Helper trait to feature gate the [`From<crate::proto::Command>`] dependency.
pub trait InternalIrcMessageOutgoingBase {}

#[cfg(not(feature = "essentials"))]
impl<T> InternalIrcMessageOutgoingBase for T {}

#[cfg(feature = "essentials")]
/// Helper trait to feature gate the [`std::borrow::Borrow<crate::proto::Message>`] dependency.
pub trait InternalIrcMessageIncomingBase: std::borrow::Borrow<crate::proto::Message> {}

#[cfg(feature = "essentials")]
impl<T> InternalIrcMessageIncomingBase for T where T: std::borrow::Borrow<crate::proto::Message> {}

#[cfg(not(feature = "essentials"))]
/// Helper trait to feature gate the [`std::borrow::Borrow<crate::proto::Message>`] dependency.
pub trait InternalIrcMessageIncomingBase {}

#[cfg(not(feature = "essentials"))]
impl<T> InternalIrcMessageIncomingBase for T {}

/// An message type that supports decoding the commands necessary to maintain communication with a basic IRC server.
pub trait InternalIrcMessageIncoming: InternalIrcMessageIncomingBase {
    // Override if you want to be able to join channels:

    /// Whether or not this message is a `RPL_ENDOFMOTD` response.
    fn is_end_of_motd(&self) -> bool {
        false
    }

    /// Whether or not this message is a `ERR_NOMOTD` response.
    fn is_err_nomotd(&self) -> bool {
        false
    }

    // Override if you don't want to time out after a few minutes:

    /// Whether or not this message is a `PONG` message.
    fn is_pong(&self) -> bool {
        false
    }

    /// If this message is a `PING` message, this returns the payload.
    fn as_ping<'a>(&'a self) -> Option<&'a str> {
        None
    }

    // Override if you want to be able to know when you've been kicked out of a server:

    /// Whether or not this message is a `QUIT` message.
    fn is_quit(&self) -> bool {
        false
    }
}

/// An message type that supports encoding enough message types to maintain communication with an IRC server.
/// All functions can be deduced automatically as long as `new_raw` is implemented.
pub trait InternalIrcMessageOutgoing: InternalIrcMessageOutgoingBase {
    /// Create a message from scratch.
    fn new_raw(command: String, arguments: Vec<String>) -> Self;

    // Basic functionality
    // Encode message types that are absolutely necessary for the communication even with basic IRC servers.

    /// Create a `PING` message.
    fn new_ping(server: String) -> Self {
        Self::new_raw("PING".to_owned(), vec![server])
    }

    /// Create a `PONG` message.
    fn new_pong(daemon: String) -> Self {
        Self::new_raw("PONG".to_owned(), vec![daemon])
    }

    /// Create a `CAP END` message (end of capabilities request).
    fn new_cap_end() -> Self {
        Self::new_raw("CAP".to_owned(), vec!["END".to_owned()])
    }

    /// Create a `NICK` message.
    fn new_nick(nick: String) -> Self {
        Self::new_raw("NICK".to_owned(), vec![nick])
    }

    /// Create a `USER` message. The user mode will be set to `0`.
    fn new_user(username: String, realname: String) -> Self {
        Self::new_raw(
            "USER".to_owned(),
            vec![username, "0".to_owned(), "*".to_owned(), realname],
        )
    }

    /// Create a `JOIN` message for channels that do not require authentification.
    fn new_join(channel_list: String) -> Self {
        Self::new_raw("JOIN".to_owned(), vec![channel_list])
    }

    /// Create a `PART` message.
    fn new_part(channel_list: String) -> Self {
        Self::new_raw("PART".to_owned(), vec![channel_list])
    }

    /// Create a `QUIT` message.
    fn new_quit(message: String) -> Self {
        Self::new_raw("QUIT".to_owned(), vec![message])
    }

    // User Authentification
    // Encode message types necessary to authenticate users.

    /// Create a `PASS` message.
    fn new_pass(password: String) -> Self {
        Self::new_raw("PASS".to_owned(), vec![password])
    }

    /// Create a `JOIN` message for channels that require authentification.
    fn new_authenticated_join(channel_list: String, pass_list: String) -> Self {
        Self::new_raw("JOIN".to_owned(), vec![channel_list, pass_list])
    }

    /// Create a `NICKSERV` message.
    fn new_nickserv(commands: Vec<String>) -> Self {
        Self::new_raw("NICKSERV".to_owned(), commands)
    }
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
    fn new_raw(command: String, arguments: Vec<String>) -> Self {
        Command::Raw(command, arguments).into()
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

    // Implementations below aren't strictly necessary since they should be equal to the default implementations.
    // However, they are included here to make sure that no new bugs are introduced.
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

    fn as_ping<'a>(&'a self) -> Option<&'a str> {
        if let Command::PING(ref payload, _) = self.command {
            Some(payload.as_ref())
        } else {
            None
        }
    }
}
