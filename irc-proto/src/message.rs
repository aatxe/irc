//! A module providing a data structure for messages to and from IRC servers.
use std::borrow::Cow;
use std::fmt;
use std::num::NonZeroU16;
use std::str::FromStr;

use chan::ChannelExt;
use command::Command;
use error;
use error::{MessageParseError, ProtocolError};
use prefix::Prefix;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Part {
    start: u16,
    end: u16,
}

impl Part {
    fn new(start: usize, end: usize) -> Part {
        Part {
            start: start as u16,
            end: end as u16,
        }
    }

    fn index<'a>(&self, s: &'a str) -> &'a str {
        &s[self.start as usize..self.end as usize]
    }
}

impl From<Command> for Message {
    fn from(c: Command) -> Message {
        unimplemented!("dummy impl")
    }
}

/// The maximum number of bytes allowed in a message, currently set to `u16::max_value()`, though
/// the IRC specification is stricter than this.
pub const MAX_BYTES: usize = u16::max_value() as usize;

/// The maximum number of arguments supported by the message parser, currently set to 15 as it is
/// the maximum according to the IRC specification.
pub const MAX_ARGS: usize = 15;

/// A parsed IRC message, containing a buffer with pointers to the individual parts.
#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    buf: String,
    tags: Option<Part>,
    prefix: Option<Part>,
    command: Part,
    args: [Part; MAX_ARGS],
    args_len: u8,
    suffix: Option<Part>,
}

impl Message {
    /// Parses the message, converting the given object into an owned string.
    ///
    /// This will perform an additional allocation if a `String` is passed. To avoid this and
    /// transfer ownership instead, use the [`parse_string`] method.
    ///
    /// # Error
    ///
    /// This method will fail in the following conditions:
    ///
    /// - The message length is longer than the maximum supported number of bytes ([`MAX_BYTES`]).
    /// - The message contains more than the maximum supported number of arguments ([`MAX_ARGS`]).
    /// - The message is missing required components such as the trailing CRLF or the command.
    ///
    /// Note that it does not check whether the parts of the message have illegal forms, as
    /// there is little benefit to restricting that. It will also not perform any allocations
    /// except for the initial buffer; any unbounded dynamically-sized components like tags are
    /// parsed through iterators instead (see the [`tags`] method).
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse("PRIVMSG #rust :Hello Rustaceans!\r\n")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`parse_string`]: #method.parse_string
    /// [`MAX_BYTES`]: ./constant.MAX_BYTES.html
    /// [`MAX_ARGS`]: ./constant.MAX_ARGS.html
    pub fn parse<S>(message: S) -> Result<Self, MessageParseError>
    where
        S: ToString,
    {
        Message::parse_string(message.to_string())
    }

    /// Takes ownership of the given string and parses it into a message.
    ///
    /// For more information about the details of the parser, see the [`parse`] method.
    ///
    /// [`parse`] #method.parse
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse_string("NICK ferris\r\n".to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_string(message: String) -> Result<Self, MessageParseError> {
        if message.len() > MAX_BYTES {
            return Err(MessageParseError::MaxLengthExceeded);
        }
        if !message.ends_with("\r\n") {
            return Err(MessageParseError::MissingCrLf);
        }
        let message_end = message.len() - '\n'.len_utf8() - '\r'.len_utf8();
        let mut i = 0;

        let mut tags = None;
        if message[i..].starts_with('@') {
            i += '@'.len_utf8();
            let start = i;

            i += message[i..].find(' ').unwrap_or_else(|| message_end - i);
            let end = i;

            tags = Some(Part::new(start, end));
        }

        while message[i..].starts_with(' ') {
            i += ' '.len_utf8();
        }

        let mut prefix = None;
        if message[i..].starts_with(':') {
            i += ':'.len_utf8();
            let start = i;

            i += message[i..].find(' ').unwrap_or_else(|| message_end - i);
            let end = i;

            prefix = Some(Part::new(start, end));
        }

        while message[i..].starts_with(' ') {
            i += ' '.len_utf8();
        }

        let command = {
            let start = i;

            i += message[i..].find(' ').unwrap_or_else(|| message_end - i);
            let end = i;

            Part::new(start, end)
        };

        if command.start == command.end {
            return Err(MessageParseError::MissingCommand);
        }

        while message[i..].starts_with(' ') {
            i += ' '.len_utf8();
        }

        let mut args = [Part::new(0, 0); MAX_ARGS];
        let mut args_len = 0;
        let mut suffix = None;

        while i < message_end {
            if message[i..].starts_with(':') {
                i += ':'.len_utf8();
                let start = i;

                i = message_end;
                let end = i;

                suffix = Some(Part::new(start, end));
                break;
            }

            if args_len as usize >= MAX_ARGS {
                return Err(MessageParseError::MaxArgsExceeded);
            }

            let start = i;

            i += message[i..].find(' ').unwrap_or_else(|| message_end - i);
            let end = i;

            args[args_len as usize] = Part::new(start, end);
            args_len += 1;

            while message[i..].starts_with(' ') {
                i += ' '.len_utf8();
            }
        }

        Ok(Message {
            buf: message,
            tags,
            prefix,
            command,
            args,
            args_len,
            suffix,
        })
    }

    /// Borrows from the string slice containing the serialized message.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let raw_message = "JOIN #rust\r\n";
    /// let parsed_message = Message::parse(raw_message)?;
    /// assert_eq!(parsed_message.as_str(), raw_message);
    /// # Ok(())
    /// # }
    pub fn as_str(&self) -> &str {
        &self.buf
    }

    /// Consumes this message, producing the string containing the serialized message.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let raw_message = "JOIN #rust\r\n";
    /// let parsed_message = Message::parse(raw_message)?;
    /// assert_eq!(parsed_message.into_string(), raw_message);
    /// # Ok(())
    /// # }
    pub fn into_string(self) -> String {
        self.buf
    }

    /// Produces a parser iterator over the message's tags. The iterator will produce items of
    /// `(&str, Option<Cow<str>>)` for each tag in order, containing the tag's key and its value if
    /// one exists for that key. It is entirely zero copy except when the value contains escape
    /// sequences, in which case the unescaped value will be produced and stored in an owned
    /// buffer.
    ///
    /// This parser will not dedupe tags, nor will it check whether the tag's key is empty or
    /// whether it contains illegal characters. 
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    /// use std::borrow::Cow;
    ///
    /// let message = Message::parse(
    ///     "@aaa=bbb;ccc;example.com/ddd=eee :nick!ident@host.com PRIVMSG me :Hello\r\n"
    /// )?;
    ///
    /// let mut tags = message.tags();
    /// assert_eq!(tags.len(), 3);
    ///
    /// assert_eq!(tags.next(), Some(("aaa", Some(Cow::Borrowed("bbb")))));
    /// assert_eq!(tags.next(), Some(("ccc", None)));
    /// assert_eq!(tags.next(), Some(("example.com/ddd", Some(Cow::Borrowed("eee")))));
    /// assert_eq!(tags.next(), None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn tags(&self) -> Tags {
        Tags {
            buf: self.tags.as_ref().map(|part| part.index(&self.buf)).unwrap_or(""),
        }
    }

    /// Returns a string slice containing the message's prefix, if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse(":nick!ident@host.com PRIVMSG me :Hello\r\n")?;
    /// assert_eq!(message.prefix(), Some("nick!ident@host.com"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_ref().map(|part| part.index(&self.buf))
    }

    /// Returns a string slice containing the message's command.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse("NICK ferris\r\n")?;
    /// assert_eq!(message.command(), "NICK");
    /// # Ok(())
    /// # }
    /// ```
    pub fn command(&self) -> &str {
        self.command.index(&self.buf)
    }

    /// Returns a string slice containing the value of the given argument (indexed starting from
    /// 0), if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse("PRIVMSG #rust :Hello Rustaceans!\r\n")?;
    /// assert_eq!(message.arg(0), Some("#rust"));
    /// assert_eq!(message.arg(1), None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn arg(&self, arg: usize) -> Option<&str> {
        if arg < self.args_len as usize {
            Some(self.args[arg].index(&self.buf))
        } else {
            None
        }
    }

    /// Returns an iterator over the message's arguments, yielding `&str` string slices containing
    /// each argument in order.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse("USER guest tolmoon tolsun :Ronnie Reagan\r\n")?;
    /// let mut args = message.args();
    /// assert_eq!(args.len(), 3);
    /// assert_eq!(args.next(), Some("guest"));
    /// assert_eq!(args.next(), Some("tolmoon"));
    /// assert_eq!(args.next(), Some("tolsun"));
    /// assert_eq!(args.next(), None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn args(&self) -> Args {
        Args {
            buf: &self.buf,
            args: self.args.iter().take(self.args_len as usize),
        }
    }

    /// Returns a string slice containing the message's suffix, if it exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse("USER guest tolmoon tolsun :Ronnie Reagan\r\n")?;
    /// assert_eq!(message.suffix(), Some("Ronnie Reagan"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn suffix(&self) -> Option<&str> {
        self.suffix.as_ref().map(|part| part.index(&self.buf))
    }
}

impl FromStr for Message {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Message::parse(s)
            .map_err(|err| ProtocolError::InvalidMessage { string: s.to_string(), cause: err })
    }
}

impl AsRef<str> for Message {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.buf)
    }
}

/// A parser iterator over a message's tags. See [`Message::tags`] for more information.
///
/// [`Message::tags`]: ./struct.Message.html#method.tags
pub struct Tags<'a> {
    buf: &'a str,
}

impl<'a> Iterator for Tags<'a> {
    type Item = (&'a str, Option<Cow<'a, str>>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() == 0 {
            None
        } else {
            let tag = self.buf
                .char_indices()
                .find(|&(_i, c)| c == ';')
                .map(|(i, _c)| &self.buf[..i])
                .unwrap_or(&self.buf);

            if self.buf.len() == tag.len() {
                self.buf = "";
            } else {
                self.buf = &self.buf[tag.len() + ';'.len_utf8()..];
            }
            
            if let Some(key_end) = tag.find('=') {
                let key = &tag[..key_end];
                let mut raw_value = &tag[key_end + '='.len_utf8()..];

                let mut value = String::new();
                while let Some(escape_idx) = raw_value.find('\\') {
                    value.push_str(&raw_value[..escape_idx]);
                    let c = match raw_value[escape_idx + '\\'.len_utf8()..].chars().next() {
                        Some(':') => Some(';'),
                        Some('s') => Some(' '),
                        Some('\\') => Some('\\'),
                        Some('r') => Some('\r'),
                        Some('n') => Some('\n'),
                        Some(c) => Some(c),
                        None => None,
                    };
                    if let Some(c) = c {
                        value.push(c);
                    }
                    raw_value = &raw_value[
                        (escape_idx
                            + '\\'.len_utf8()
                            + c.map(char::len_utf8).unwrap_or(0)
                        )..
                    ];
                }
                if value.len() == 0 {
                    Some((key, Some(Cow::Borrowed(raw_value))))
                } else {
                    value.push_str(raw_value);
                    Some((key, Some(Cow::Owned(value))))
                }
            } else {
                Some((tag, None))
            }
        }
    }
}

impl<'a> ExactSizeIterator for Tags<'a> {
    fn len(&self) -> usize {
        if self.buf.len() == 0 {
            0
        } else {
            self.buf.chars().filter(|&c| c == ';').count() + 1
        }
    }
}

/// An iterator over a message's tags. See [`Message::args`] for more information.
///
/// [`Message::args`]: ./struct.Message.html#method.args
pub struct Args<'a> {
    buf: &'a str,
    args: std::iter::Take<std::slice::Iter<'a, Part>>,
}

impl<'a> Iterator for Args<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.args.next().map(|part| part.index(self.buf))
    }
}

impl<'a> ExactSizeIterator for Args<'a> {
    fn len(&self) -> usize {
        self.args.len()
    }
}

#[cfg(test)]
mod test {



    // Legacy tests
    // TODO: Adapt to new message/command API

    /*
    use super::{Message, Tag};
    use command::Command::{PRIVMSG, QUIT, Raw};

    #[test]
    #[ignore]
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
    #[ignore]
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
    #[ignore]
    fn to_string() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!(&message.to_string()[..], "PRIVMSG test :Testing!\r\n");
        let message = Message {
            tags: None,
            prefix: Some("test!test@test".into()),
            command: PRIVMSG(format!("test"), format!("Still testing!")),
        };
        assert_eq!(
            &message.to_string()[..],
            ":test!test@test PRIVMSG test :Still testing!\r\n"
        );
    }

    #[test]
    #[ignore]
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
            prefix: Some("test!test@test".into()),
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
            prefix: Some("test!test@test".into()),
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
    #[ignore]
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
    #[ignore]
    fn from_and_to_string() {
        let message = "@aaa=bbb;ccc;example.com/ddd=eee :test!test@test PRIVMSG test :Testing with \
                       tags!\r\n";
        assert_eq!(message.parse::<Message>().unwrap().to_string(), message);
    }

    #[test]
    #[ignore]
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
            prefix: Some("test!test@test".into()),
            command: PRIVMSG(format!("test"), format!("Still testing!")),
        };
        let msg: Message = ":test!test@test PRIVMSG test :Still testing!\r\n".into();
        assert_eq!(msg, message);
    }

    #[test]
    #[ignore]
    fn to_message_with_colon_in_arg() {
        // Apparently, UnrealIRCd (and perhaps some others) send some messages that include
        // colons within individual parameters. So, let's make sure it parses correctly.
        let message = Message {
            tags: None,
            prefix: Some("test!test@test".into()),
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
    #[ignore]
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
    #[ignore]
    #[should_panic]
    fn to_message_invalid_format() {
        let _: Message = ":invalid :message".into();
    }
    */
}
