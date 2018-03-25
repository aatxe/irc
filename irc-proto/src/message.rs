//! A module providing a data structure for messages to and from IRC servers.
use std::borrow::ToOwned;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use error;
use error::{ProtocolError, MessageParseError};
use chan::ChannelExt;
use command::{Command, OwnedCommand};

/// A data structure representing an IRC message according to the protocol specification. It
/// consists of a collection of IRCv3 tags, a prefix (describing the source of the message), and
/// the protocol command. If the command is unknown, it is treated as a special raw command that
/// consists of a collection of arguments and the special suffix argument. Otherwise, the command
/// is parsed into a more useful form as described in [Command](../command/enum.Command.html).
#[derive(Clone, PartialEq, Debug)]
pub struct OwnedMessage {
    /// Message tags as defined by [IRCv3.2](http://ircv3.net/specs/core/message-tags-3.2.html).
    /// These tags are used to add extended information to the given message, and are commonly used
    /// in IRCv3 extensions to the IRC protocol.
    pub tags: Option<Vec<OwnedTag>>,
    /// The message prefix (or source) as defined by [RFC 2812](http://tools.ietf.org/html/rfc2812).
    pub prefix: Option<String>,
    /// The IRC command, parsed according to the known specifications. The command itself and its
    /// arguments (including the special suffix argument) are captured in this component.
    pub command: OwnedCommand,
}

/// A data structure representing an IRC message according to the protocol specification. It
/// consists of a collection of IRCv3 tags, a prefix (describing the source of the message), and
/// the protocol command. If the command is unknown, it is treated as a special raw command that
/// consists of a collection of arguments and the special suffix argument. Otherwise, the command
/// is parsed into a more useful form as described in [Command](../command/enum.Command.html).
#[derive(Clone, PartialEq, Debug)]
pub struct Message<'msg> {
    /// Message tags as defined by [IRCv3.2](http://ircv3.net/specs/core/message-tags-3.2.html).
    /// These tags are used to add extended information to the given message, and are commonly used
    /// in IRCv3 extensions to the IRC protocol.
    pub tags: Option<Vec<Tag<'msg>>>,
    /// The message prefix (or source) as defined by [RFC 2812](http://tools.ietf.org/html/rfc2812).
    pub prefix: Option<&'msg str>,
    /// The IRC command, parsed according to the known specifications. The command itself and its
    /// arguments (including the special suffix argument) are captured in this component.
    pub command: Command<'msg>,
}

impl<'msg> Message<'msg> {
    /// Creates a new borrowed message from the given components.
    ///
    /// # Example
    /// ```
    /// # extern crate irc_proto;
    /// # use irc_proto::Message;
    /// # fn main() {
    /// let message = Message::new(
    ///     Some("nickname!username@hostname"), "JOIN", &["#channel"], None
    /// ).unwrap();
    /// # }
    /// ```
    pub fn new<'a>(
        prefix: Option<&'a str>,
        command: &'a str,
        args: &[&'a str],
        suffix: Option<&'a str>
    ) -> Result<Message<'a>, MessageParseError> {
        Message::with_tags(None, prefix, command, args, suffix)
    }

    /// Creates a new IRCv3.2 borrowed message from the given components. This includes IRCv3 tags
    /// which are used to add extended information to the given message, and are commonly used
    /// throughout the IRCv3 protocol extensions.
    pub fn with_tags<'a>(
        tags: Option<Vec<Tag<'a>>>,
        prefix: Option<&'a str>,
        command: &'a str,
        args: &[&'a str],
        suffix: Option<&'a str>
    ) -> Result<Message<'a>, MessageParseError> {
        Ok(Message {
            tags: tags.map(|tags| tags.to_owned()),
            prefix: prefix,
            command: Command::new(command, args, suffix)?,
        })
    }

    /// Gets the nickname of the message source, if it exists.
    ///
    /// # Example
    /// ```
    /// # extern crate irc_proto;
    /// # use irc_proto::Message;
    /// # fn main() {
    /// let message = Message::new(
    ///     Some("nickname!username@hostname"), "JOIN", vec!["#channel"], None
    /// ).unwrap();
    /// assert_eq!(message.source_nickname(), Some("nickname"));
    /// # }
    /// ```
    pub fn source_nickname(&self) -> Option<&str> {
        // <prefix> ::= <servername> | <nick> [ '!' <user> ] [ '@' <host> ]
        // <servername> ::= <host>
        self.prefix.and_then(|s| match (
            s.find('!'),
            s.find('@'),
            s.find('.'),
        ) {
            (Some(i), _, _) | // <nick> '!' <user> [ '@' <host> ]
            (None, Some(i), _) => Some(&s[..i]), // <nick> '@' <host>
            (None, None, None) => Some(s), // <nick>
            _ => None, // <servername>
        })
    }

    /// Gets the likely intended place to respond to this message.
    /// If the type of the message is a `PRIVMSG` or `NOTICE` and the message is sent to a channel,
    /// the result will be that channel. In all other cases, this will call `source_nickname`.
    ///
    /// # Example
    /// ```
    /// # extern crate irc_proto;
    /// # use irc_proto::Message;
    /// # fn main() {
    /// let msg1 = Message::new(
    ///     Some("ada"), "PRIVMSG", &["#channel"], Some("Hi, everyone!")
    /// ).unwrap();
    /// assert_eq!(msg1.response_target(), Some("#channel"));
    /// let msg2 = Message::new(
    ///     Some("ada"), "PRIVMSG", &["betsy"], Some("betsy: hi")
    /// ).unwrap();
    /// assert_eq!(msg2.response_target(), Some("ada"));
    /// # }
    /// ```
    pub fn response_target(&self) -> Option<&str> {
        match self.command {
            Command::PRIVMSG(target, _) if target.is_channel_name() => Some(target),
            Command::NOTICE(target, _) if target.is_channel_name() => Some(target),
            _ => self.source_nickname()
        }
    }

    /// Converts a OwnedMessage into a String according to the IRC protocol.
    ///
    /// # Example
    /// ```
    /// # extern crate irc_proto;
    /// # use irc_proto::Message;
    /// # fn main() {
    /// let msg = Message::new(
    ///     Some("ada"), "PRIVMSG", &["#channel"], Some("Hi, everyone!")
    /// ).unwrap();
    /// assert_eq!(msg.to_string(), ":ada PRIVMSG #channel :Hi, everyone!\r\n");
    /// # }
    /// ```
    pub fn to_string(&self) -> String {
        let mut ret = String::new();
        if let Some(ref tags) = self.tags {
            ret.push('@');
            for tag in tags {
                ret.push_str(&tag.0);
                if let Some(ref value) = tag.1 {
                    ret.push('=');
                    ret.push_str(value);
                }
                ret.push(';');
            }
            ret.pop();
            ret.push(' ');
        }
        if let Some(ref prefix) = self.prefix {
            ret.push(':');
            ret.push_str(prefix);
            ret.push(' ');
        }
        let cmd: String = String::from(self.command.clone());
        ret.push_str(&cmd);
        ret.push_str("\r\n");
        ret
    }

    pub fn from_str<'a>(s: &'a str) -> Result<Message<'a>, ProtocolError> {
        if s.is_empty() {
            return Err(ProtocolError::InvalidMessage {
                string: s.to_owned(),
                cause: MessageParseError::EmptyMessage,
            })
        }

        let mut state = s;

        let tags = if state.starts_with('@') {
            let tags = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i + 1..]);
            tags.map(|ts| {
                ts.split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s: &str| {
                        let mut iter = s.splitn(2, '=');
                        let (fst, snd) = (iter.next(), iter.next());
                        Tag(fst.unwrap_or(""), snd)
                    })
                    .collect::<Vec<_>>()
            })
        } else {
            None
        };

        let prefix = if state.starts_with(':') {
            let prefix = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i + 1..]);
            prefix
        } else {
            None
        };

        let line_ending_len = if state.ends_with("\r\n") {
            "\r\n"
        } else if state.ends_with('\r') {
            "\r"
        } else if state.ends_with('\n') {
            "\n"
        } else {
            ""
        }.len();

        let suffix = if state.contains(" :") {
            let suffix = state.find(" :").map(|i| &state[i + 2..state.len() - line_ending_len]);
            state = state.find(" :").map_or("", |i| &state[..i + 1]);
            suffix
        } else {
            state = &state[..state.len() - line_ending_len];
            None
        };

        let command = match state.find(' ').map(|i| &state[..i]) {
            Some(cmd) => {
                state = state.find(' ').map_or("", |i| &state[i + 1..]);
                cmd
            }
            // If there's no arguments but the "command" starts with colon, it's not a command.
            None if state.starts_with(':') => return Err(ProtocolError::InvalidMessage {
                string: s.to_owned(),
                cause: MessageParseError::InvalidCommand,
            }),
            // If there's no arguments following the command, the rest of the state is the command.
            None => {
                let cmd = state;
                state = "";
                cmd
            },
        };

        let args: Vec<_> = state.splitn(14, ' ').filter(|s| !s.is_empty()).collect();

        Message::with_tags(tags, prefix, command, &args, suffix).map_err(|e| {
            ProtocolError::InvalidMessage {
                string: s.to_owned(),
                cause: e,
            }
        })
    }
}

impl OwnedMessage {
    /// Creates a new message from the given components, allocating copies in the process.
    ///
    /// # Example
    /// ```
    /// # extern crate irc_proto;
    /// # use irc_proto::OwnedMessage;
    /// # fn main() {
    /// let message = OwnedMessage::new(
    ///     Some("nickname!username@hostname"), "JOIN", vec!["#channel"], None
    /// ).unwrap();
    /// # }
    /// ```
    pub fn new(
        prefix: Option<&str>,
        command: &str,
        args: Vec<&str>,
        suffix: Option<&str>,
    ) -> Result<OwnedMessage, MessageParseError> {
        OwnedMessage::with_tags(None, prefix, command, args, suffix)
    }

    /// Creates a new IRCv3.2 message from the given components, allocating copies in the process.
    /// This includes IRCv3 tags which are used to add extended information to the given message,
    /// and are commonly used throughout the IRCv3 protocol extensions.
    pub fn with_tags(
        tags: Option<Vec<OwnedTag>>,
        prefix: Option<&str>,
        command: &str,
        args: Vec<&str>,
        suffix: Option<&str>,
    ) -> Result<OwnedMessage, error::MessageParseError> {
        Ok(OwnedMessage {
            tags: tags,
            prefix: prefix.map(|s| s.to_owned()),
            command: Command::new(command, &args, suffix)?,
        })
    }
}

impl From<OwnedCommand> for OwnedMessage {
    fn from(cmd: OwnedCommand) -> OwnedMessage {
        OwnedMessage {
            tags: None,
            prefix: None,
            command: cmd,
        }
    }
}

impl FromStr for OwnedMessage {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<OwnedMessage, Self::Err> {
        if s.is_empty() {
            return Err(ProtocolError::InvalidMessage {
                string: s.to_owned(),
                cause: MessageParseError::EmptyMessage,
            })
        }

        let mut state = s;

        let tags = if state.starts_with('@') {
            let tags = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i + 1..]);
            tags.map(|ts| {
                ts.split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s: &str| {
                        let mut iter = s.splitn(2, '=');
                        let (fst, snd) = (iter.next(), iter.next());
                        OwnedTag(fst.unwrap_or("").to_owned(), snd.map(|s| s.to_owned()))
                    })
                    .collect::<Vec<_>>()
            })
        } else {
            None
        };

        let prefix = if state.starts_with(':') {
            let prefix = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i + 1..]);
            prefix
        } else {
            None
        };

        let line_ending_len = if state.ends_with("\r\n") {
            "\r\n"
        } else if state.ends_with('\r') {
            "\r"
        } else if state.ends_with('\n') {
            "\n"
        } else {
            ""
        }.len();

        let suffix = if state.contains(" :") {
            let suffix = state.find(" :").map(|i| &state[i + 2..state.len() - line_ending_len]);
            state = state.find(" :").map_or("", |i| &state[..i + 1]);
            suffix
        } else {
            state = &state[..state.len() - line_ending_len];
            None
        };

        let command = match state.find(' ').map(|i| &state[..i]) {
            Some(cmd) => {
                state = state.find(' ').map_or("", |i| &state[i + 1..]);
                cmd
            }
            // If there's no arguments but the "command" starts with colon, it's not a command.
            None if state.starts_with(':') => return Err(ProtocolError::InvalidMessage {
                string: s.to_owned(),
                cause: MessageParseError::InvalidCommand,
            }),
            // If there's no arguments following the command, the rest of the state is the command.
            None => {
                let cmd = state;
                state = "";
                cmd
            },
        };

        let args: Vec<_> = state.splitn(14, ' ').filter(|s| !s.is_empty()).collect();

        OwnedMessage::with_tags(tags, prefix, command, args, suffix).map_err(|e| {
            ProtocolError::InvalidMessage {
                string: s.to_owned(),
                cause: e,
            }
        })
    }
}

impl<'a> Display for Message<'a> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.to_string())
    }
}

/// A message tag as defined by [IRCv3.2](http://ircv3.net/specs/core/message-tags-3.2.html).
/// It consists of a tag key, and an optional value for the tag. Each message can contain a number
/// of tags (in the string format, they are separated by semicolons). OwnedTags are used to add extended
/// information to a message under IRCv3.
#[derive(Clone, PartialEq, Debug)]
pub struct OwnedTag(pub String, pub Option<String>);

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Tag<'tag>(pub &'tag str, pub Option<&'tag str>);

#[cfg(test)]
mod test {
    use super::{Message, Tag};
    use command::Command::{PRIVMSG, QUIT, Raw};

    #[test]
    fn new() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!(
            Message::new(None, "PRIVMSG", vec!["test"], Some("Testing!")).unwrap(),
            message
        )
    }

    #[test]
    fn source_nickname() {
        assert_eq!(
            Message::new(None, "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            None
        );

        assert_eq!(
            Message::new(Some("irc.test.net"), "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            None
        );

        assert_eq!(
            Message::new(Some("test!test@test"), "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            Some("test")
        );

        assert_eq!(
            Message::new(Some("test@test"), "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            Some("test")
        );

        assert_eq!(
            Message::new(Some("test!test@irc.test.com"), "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            Some("test")
        );

        assert_eq!(
            Message::new(Some("test!test@127.0.0.1"), "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            Some("test")
        );

        assert_eq!(
            Message::new(Some("test@test.com"), "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            Some("test")
        );

        assert_eq!(
            Message::new(Some("test"), "PING", vec![], Some("data"))
                .unwrap()
                .source_nickname(),
            Some("test")
        );
    }

    #[test]
    fn to_string() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!(&message.to_string()[..], "PRIVMSG test :Testing!\r\n");
        let message = Message {
            tags: None,
            prefix: Some(format!("test!test@test")),
            command: PRIVMSG(format!("test"), format!("Still testing!")),
        };
        assert_eq!(
            &message.to_string()[..],
            ":test!test@test PRIVMSG test :Still testing!\r\n"
        );
    }

    #[test]
    fn from_string() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!(
            "PRIVMSG test :Testing!\r\n".parse::<Message>().unwrap(),
            message
        );
        let message = Message {
            tags: None,
            prefix: Some(format!("test!test@test")),
            command: PRIVMSG(format!("test"), format!("Still testing!")),
        };
        assert_eq!(
            ":test!test@test PRIVMSG test :Still testing!\r\n"
                .parse::<Message>()
                .unwrap(),
            message
        );
        let message = Message {
            tags: Some(vec![
                Tag(format!("aaa"), Some(format!("bbb"))),
                Tag(format!("ccc"), None),
                Tag(format!("example.com/ddd"), Some(format!("eee"))),
            ]),
            prefix: Some(format!("test!test@test")),
            command: PRIVMSG(format!("test"), format!("Testing with tags!")),
        };
        assert_eq!(
            "@aaa=bbb;ccc;example.com/ddd=eee :test!test@test PRIVMSG test :Testing with \
                    tags!\r\n"
                .parse::<Message>()
                .unwrap(),
            message
        )
    }

    #[test]
    fn from_string_atypical_endings() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!(
            "PRIVMSG test :Testing!\r".parse::<Message>().unwrap(),
            message
        );
        assert_eq!(
            "PRIVMSG test :Testing!\n".parse::<Message>().unwrap(),
            message
        );
        assert_eq!(
            "PRIVMSG test :Testing!".parse::<Message>().unwrap(),
            message
        );
    }

    #[test]
    fn from_and_to_string() {
        let message = "@aaa=bbb;ccc;example.com/ddd=eee :test!test@test PRIVMSG test :Testing with \
                       tags!\r\n";
        assert_eq!(message.parse::<Message>().unwrap().to_string(), message);
    }

    #[test]
    fn to_message() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        let msg: Message = "PRIVMSG test :Testing!\r\n".into();
        assert_eq!(msg, message);
        let message = Message {
            tags: None,
            prefix: Some(format!("test!test@test")),
            command: PRIVMSG(format!("test"), format!("Still testing!")),
        };
        let msg: Message = ":test!test@test PRIVMSG test :Still testing!\r\n".into();
        assert_eq!(msg, message);
    }

    #[test]
    fn to_message_with_colon_in_arg() {
        // Apparently, UnrealIRCd (and perhaps some others) send some messages that include
        // colons within individual parameters. So, let's make sure it parses correctly.
        let message = Message {
            tags: None,
            prefix: Some(format!("test!test@test")),
            command: Raw(
                format!("COMMAND"),
                vec![format!("ARG:test")],
                Some(format!("Testing!")),
            ),
        };
        let msg: Message = ":test!test@test COMMAND ARG:test :Testing!\r\n".into();
        assert_eq!(msg, message);
    }

    #[test]
    fn to_message_no_prefix_no_args() {
        let message = Message {
            tags: None,
            prefix: None,
            command: QUIT(None),
        };
        let msg: Message = "QUIT\r\n".into();
        assert_eq!(msg, message);
    }

    #[test]
    #[should_panic]
    fn to_message_invalid_format() {
        let _: Message = ":invalid :message".into();
    }
}
