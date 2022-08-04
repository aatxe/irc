//! Support for the IRC protocol using Tokio.

#![warn(missing_docs)]

pub mod caps;
pub mod chan;
pub mod colors;
pub mod command;
pub mod error;
#[cfg(feature = "tokio")]
pub mod irc;
#[cfg(feature = "tokio")]
pub mod line;
pub mod message;
pub mod mode;
pub mod prefix;
pub mod response;

use irc_interface::{InternalIrcMessageIncoming, InternalIrcMessageOutgoing, MessageCodec};

pub use self::caps::{Capability, NegotiationVersion};
pub use self::chan::ChannelExt;
pub use self::colors::FormattedStringExt;
pub use self::command::{BatchSubCommand, CapSubCommand, Command};
#[cfg(feature = "tokio")]
pub use self::irc::IrcCodec;
pub use self::message::Message;
pub use self::mode::{ChannelMode, Mode, UserMode};
pub use self::prefix::Prefix;
pub use self::response::Response;

#[cfg(feature = "tokio")]
impl MessageCodec for IrcCodec {
    type MsgItem = Message;
    type Error = error::ProtocolError;

    fn try_new(char_encoding: impl AsRef<str>) -> Result<Self, <Self as MessageCodec>::Error> {
        Self::new(char_encoding.as_ref())
    }
}

#[cfg(feature = "tokio")]
impl InternalIrcMessageOutgoing for Message {
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

#[cfg(feature = "tokio")]
impl InternalIrcMessageIncoming for Message {
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
