//! A trait that describes a codec that can be used to encode or decode IRC messages.
//! At the moment, this is just a wrapper around [`tokio_util::codec::Decoder`] and [`tokio_util::codec::Encoder`].

use std::fmt::{Debug, Display};

pub use line::LineCodec;
pub use tokio_util::codec::{Decoder, Encoder, Framed};

pub mod line;

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

    type Error: Debug;

    /// Construct an instance of the codec based on the given character encoding.
    fn try_new(char_encoding: impl AsRef<str>) -> Result<Self, <Self as MessageCodec>::Error>;

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

/// An message type that supports decoding the commands necessary to maintain communication with a basic IRC server.
pub trait InternalIrcMessageIncoming: Sized {
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
pub trait InternalIrcMessageOutgoing: Sized {
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
