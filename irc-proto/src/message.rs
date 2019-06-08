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

/// A parsed IRC message, containing a buffer with pointers to the individual parts.
#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    buf: String,
    tags: Option<Part>,
    prefix: Option<Part>,
    command: Part,
    middle_params: Part,
    trailing_param: Option<Part>,
}

impl Message {
    /// Parses the message, converting the given object into an owned string.
    ///
    /// This will allocate a new `String` to hold the message data, even if a `String` is
    /// passed. To avoid this and transfer ownership instead, use the [`parse_string`] method.
    ///
    /// This function does not parse parameters or tags, as those may have an arbitrary number of
    /// elements and would require additional allocations to hold their pointer data. They have
    /// their own iterator-parsers that produce the elements while avoiding additional allocations;
    /// see the [`params`] and [`tags`] methods for more information.
    ///
    /// # Error
    ///
    /// This method will fail in the following conditions:
    ///
    /// - The message length is longer than the maximum supported number of bytes ([`MAX_BYTES`]).
    /// - The message is missing required components such as the trailing CRLF or the command.
    ///
    /// Note that it does not check whether the parts of the message have illegal forms, as
    /// there is little benefit to restricting that.
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
    /// [`params`]: #method.params
    /// [`tags`]: #method.tags
    /// [`MAX_BYTES`]: ./constant.MAX_BYTES.html
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
        // To make sure pointers don't overflow:
        if message.len() > MAX_BYTES {
            return Err(MessageParseError::MaxLengthExceeded);
        }

        // Make sure the message is terminated with line endings:
        if !message.ends_with("\r\n") {
            return Err(MessageParseError::MissingCrLf);
        }
        // Used as the end of the "useful" part of the message.
        let crlf = message.len() - '\n'.len_utf8() - '\r'.len_utf8();

        // Accumulating pointer used to keep track of how much has already been parsed.
        let mut i = 0;

        // If word starts with '@', it is a tag.
        let tags;
        if message[i..].starts_with('@') {
            // Take everything between '@' and next space.
            i += '@'.len_utf8();
            let start = i;

            i += message[i..].find(' ').unwrap_or_else(|| crlf - i);
            let end = i;

            tags = Some(Part::new(start, end));
        } else {
            tags = None;
        }

        // Skip to next non-space.
        while message[i..].starts_with(' ') {
            i += ' '.len_utf8();
        }

        // If word starts with ':', it is a prefix.
        let prefix;
        if message[i..].starts_with(':') {
            // Take everything between ':' and next space.
            i += ':'.len_utf8();
            let start = i;

            i += message[i..].find(' ').unwrap_or_else(|| crlf - i);
            let end = i;

            prefix = Some(Part::new(start, end));
        } else {
            prefix = None;
        }

        // Skip to next non-space.
        while message[i..].starts_with(' ') {
            i += ' '.len_utf8();
        }

        // Next word must be command.
        let command = {
            // Take everything between here and next space.
            let start = i;

            i += message[i..].find(' ').unwrap_or_else(|| crlf - i);
            let end = i;

            Part::new(start, end)
        };

        // Command must not be empty.
        if command.start == command.end {
            return Err(MessageParseError::MissingCommand);
        }

        // Skip to next non-space.
        while message[i..].starts_with(' ') {
            i += ' '.len_utf8();
        }

        // Everything from here to crlf must be parameters.
        let middle_params;
        let trailing_param;

        // If " :" exists in the remaining data, the first instance marks the beginning of a
        // trailing parameter.
        if let Some(trailing_idx) = message[i..].find(" :") {
            // Middle parameters are everything from the current position to the last
            // non-space character before the trailing parameter.
            let start = i;

            // Walking back to the last non-space character:
            let mut j = i + trailing_idx;
            while message[..j].ends_with(' ') {
                j -= ' '.len_utf8();
            }
            let end = j;
            middle_params = Part::new(start, end);

            // Trailing parameter is everything between the leading " :" and crlf.
            i += trailing_idx + ' '.len_utf8() + ':'.len_utf8();
            let start = i;
            i = crlf;
            let end = i;
            trailing_param = Some(Part::new(start, end));
        } else {
            // Middle parameters are everything from the current position to the last non-space
            // character before crlf.
            let start = i;

            // Walking back to the last non-space character:
            let mut j = crlf;
            while message[..j].ends_with(' ') {
                j -= ' '.len_utf8();
            }
            let end = j;
            middle_params = Part::new(start, end);

            // Trailing parameter does not exist:
            trailing_param = None;
        }

        // Done parsing.
        Ok(Message {
            buf: message,
            tags,
            prefix,
            command,
            middle_params,
            trailing_param,
        })
    }

    /// Returns a borrowed string slice containing the serialized message.
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

    /// Consumes this message, producing the inner string that contains the serialized message.
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
    /// one exists for that key. It is mostly zero-copy, borrowing in all cases except when the
    /// value contains escape sequences, in which case the unescaped value will be produced and
    /// stored in an owned buffer.
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
            remaining: self.tags.as_ref().map(|part| part.index(&self.buf)).unwrap_or(""),
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

    /// Returns a parser iterator over the message's parameters. The iterator will produce items of 
    /// `&str` for each parameter in order, containing the raw data in the parameter. It is entirely
    /// zero-copy, borrowing each parameter slice directly from the message buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), irc_proto::error::MessageParseError> {
    /// use irc_proto::Message;
    ///
    /// let message = Message::parse("USER guest tolmoon tolsun :Ronnie Reagan\r\n")?;
    /// let mut params = message.params();
    /// assert_eq!(params.len(), 4);
    /// assert_eq!(params.next(), Some("guest"));
    /// assert_eq!(params.next(), Some("tolmoon"));
    /// assert_eq!(params.next(), Some("tolsun"));
    /// assert_eq!(params.next(), Some("Ronnie Reagan"));
    /// assert_eq!(params.next(), None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn params(&self) -> Params {
        Params {
            remaining: self.middle_params.index(&self.buf),
            trailing: self.trailing_param.map(|part| part.index(&self.buf)),
        }
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
    remaining: &'a str,
}

impl<'a> Iterator for Tags<'a> {
    type Item = (&'a str, Option<Cow<'a, str>>);

    fn next(&mut self) -> Option<Self::Item> {
        // If remaining is empty, nothing is left to yield.
        if self.remaining.len() == 0 {
            None
        } else {
            // Take everything from here to next ';'.
            let tag = self.remaining
                .char_indices()
                .find(|&(_i, c)| c == ';')
                .map(|(i, _c)| &self.remaining[..i])
                .unwrap_or(&self.remaining);

            // Remove taken data from the remaining buffer.
            if self.remaining.len() == tag.len() {
                self.remaining = "";
            } else {
                self.remaining = &self.remaining[tag.len() + ';'.len_utf8()..];
            }
            
            // If an equal sign exists in the tag data, it must have an associated value.
            if let Some(key_end) = tag.find('=') {
                // Everything before the first equal sign is the key.
                let key = &tag[..key_end];

                // Everything after the first equal sign is the value.
                let mut raw_value = &tag[key_end + '='.len_utf8()..];

                // Resolve escape sequences if any are found.
                // This will not allocate unless data is given to it.
                let mut value = String::new();
                while let Some(escape_idx) = raw_value.find('\\') {
                    // Copy everything before this escape sequence.
                    value.push_str(&raw_value[..escape_idx]);
                    // Resolve this escape sequence.
                    let c = match raw_value[escape_idx + '\\'.len_utf8()..].chars().next() {
                        Some(':') => Some(';'),
                        Some('s') => Some(' '),
                        Some('\\') => Some('\\'),
                        Some('r') => Some('\r'),
                        Some('n') => Some('\n'),
                        Some(c) => Some(c),
                        None => None,
                    };
                    // If it resolves to a character, then push it.
                    if let Some(c) = c {
                        value.push(c);
                    }
                    // Cut off the beginning of raw_value such that it only contains
                    // everything after the parsed escape sequence.
                    // Upon looping, it will start searching from this point, skipping the last
                    // escape sequence.
                    raw_value = &raw_value[
                        (escape_idx
                            + '\\'.len_utf8()
                            + c.map(char::len_utf8).unwrap_or(0)
                        )..
                    ];
                }

                // If we didn't add data, no escape sequences exist and the raw value can be
                // referenced.
                if value.len() == 0 {
                    Some((key, Some(Cow::Borrowed(raw_value))))
                } else {
                    // Make sure you add the rest of the raw value that doesn't contain escapes.
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
        // Number of tags yielded is number of remaining semicolons plus one, unless the
        // remaining buffer is empty.
        if self.remaining.len() == 0 {
            0
        } else {
            self.remaining.chars().filter(|&c| c == ';').count() + 1
        }
    }
}

/// An iterator over a message's parameters. See [`Message::params`] for more information.
///
/// [`Message::params`]: ./struct.Message.html#method.params
pub struct Params<'a> {
    remaining: &'a str,
    trailing: Option<&'a str>,
}

impl<'a> Iterator for Params<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // If remaining slice is non-empty, we still have middle params to take:
        if self.remaining.len() > 0 {
            // Next param is everything from here to next whitespace character (or end of string).
            let param_end = self.remaining.find(' ').unwrap_or(self.remaining.len());
            let param = &self.remaining[..param_end];

            // Trim this param and its trailing spaces out of remaining.
            self.remaining = self.remaining[param_end..].trim_start_matches(' ');

            Some(param)
        } else {
            // No more middle params to parse, return trailing if it hasn't been already.
            // take will replace with None on the first call, so all future calls will return None.
            self.trailing.take()
        }
    }
}

impl<'a> ExactSizeIterator for Params<'a> {
    fn len(&self) -> usize {
        // Number of middle parameter remaining is equal to the number of points where a non-space
        // character is preceded by a space character or the beginning of the string.
        let mut middle_len = 0;
        let mut last = true;
        for c in self.remaining.chars() {
            let current = c == ' ';
            if (last, current) == (true, false) {
                middle_len += 1;
            }
            last = current;
        }

        // Add one if the trailing parameter hasn't been taken.
        let trailing_len;
        if self.trailing.is_some() {
            trailing_len = 1;
        } else {
            trailing_len = 0;
        }
        middle_len + trailing_len
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
