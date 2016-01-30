//! Messages to and from the server.
use std::borrow::ToOwned;
use std::io::{Result as IoResult};
use std::str::FromStr;
use client::data::Command;

/// IRC Message data.
#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    /// Message tags as defined by [IRCv3.2](http://ircv3.net/specs/core/message-tags-3.2.html).
    pub tags: Option<Vec<Tag>>,
    /// The message prefix (or source) as defined by [RFC 2812](http://tools.ietf.org/html/rfc2812).
    pub prefix: Option<String>,
    /// The IRC command.
    pub command: Command,
}

impl Message {
    /// Creates a new Message.
    pub fn new(prefix: Option<&str>, command: &str, args: Vec<&str>, suffix: Option<&str>)
        -> IoResult<Message> {
        Message::with_tags(None, prefix, command, args, suffix)
    }

    /// Creates a new Message optionally including IRCv3.2 message tags.
    pub fn with_tags(tags: Option<Vec<Tag>>, prefix: Option<&str>, command: &str,
                     args: Vec<&str>, suffix: Option<&str>) -> IoResult<Message> {
        Ok(Message {
            tags: tags,
            prefix: prefix.map(|s| s.to_owned()),
            command: try!(Command::new(command, args, suffix)),
        })
    }

    /// Gets the nickname of the message source, if it exists.
    pub fn get_source_nickname(&self) -> Option<&str> {
        self.prefix.as_ref().and_then(|s|
            match (s.find('!'), s.find('@'), s.find('.')) {
                (_, _, Some(_)) => None,
                (Some(i), _, None) => Some(&s[..i]),
                (None, Some(i), None) => Some(&s[..i]),
                (None, None, None) => Some(&s)
            }
        )
    }

    /// Converts a Message into a String according to the IRC protocol.
    pub fn into_string(&self) -> String {
        // TODO: tags
        let mut ret = String::new();
        if let Some(ref prefix) = self.prefix {
            ret.push(':');
            ret.push_str(&prefix);
            ret.push(' ');
        }
        let cmd: String = From::from(&self.command);
        ret.push_str(&cmd);
        ret.push_str("\r\n");
        ret
    }
}

impl From<Command> for Message {
    fn from(cmd: Command) -> Message {
        Message { tags: None, prefix: None, command: cmd }
    }
}

impl FromStr for Message {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Message, &'static str> {
        let mut state = s.clone();
        if s.len() == 0 { return Err("Cannot parse an empty string as a message.") }
        let tags = if state.starts_with("@") {
            let tags = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i+1..]);
            tags.map(|ts| ts.split(";").filter(|s| s.len() != 0).map(|s: &str| {
                let mut iter = s.splitn(2, "=");
                let (fst, snd) = (iter.next(), iter.next());
                Tag(fst.unwrap_or("").to_owned(), snd.map(|s| s.to_owned()))
            }).collect::<Vec<_>>())
        } else {
            None
        };
        let prefix = if state.starts_with(":") {
            let prefix = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i+1..]);
            prefix
        } else {
            None
        };
        let suffix = if state.contains(" :") {
            let suffix = state.find(" :").map(|i| &state[i+2..state.len()-2]);
            state = state.find(" :").map_or("", |i| &state[..i+1]);
            suffix
        } else {
            None
        };
        let command = match state.find(' ').map(|i| &state[..i]) {
            Some(cmd) => {
                state = state.find(' ').map_or("", |i| &state[i+1..]);
                cmd
            }
            _ => return Err("Cannot parse a message without a command.")
        };
        if suffix.is_none() { state = &state[..state.len() - 2] }
        let args: Vec<_> = state.splitn(14, ' ').filter(|s| s.len() != 0).collect();
        Message::with_tags(
            tags, prefix, command, args, suffix
        ).map_err(|_| "Invalid input for Command.")
    }
}

impl<'a> From<&'a str> for Message {
    fn from(s: &'a str) -> Message {
        s.parse().unwrap()
    }
}

/// A message tag as defined by [IRCv3.2](http://ircv3.net/specs/core/message-tags-3.2.html).
#[derive(Clone, PartialEq, Debug)]
pub struct Tag(String, Option<String>);

#[cfg(test)]
mod test {
    use super::{Message, Tag};
    use client::data::Command::{PRIVMSG, Raw};

    #[test]
    fn new() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!(Message::new(None, "PRIVMSG", vec!["test"], Some("Testing!")).unwrap(), message)
    }

    #[test]
    fn get_source_nickname() {
        assert_eq!(Message::new(
            None, "PING", vec![], Some("data")
        ).unwrap().get_source_nickname(), None);
        assert_eq!(Message::new(
            Some("irc.test.net"), "PING", vec![], Some("data")
        ).unwrap().get_source_nickname(), None);
        assert_eq!(Message::new(
            Some("test!test@test"), "PING", vec![], Some("data")
        ).unwrap().get_source_nickname(), Some("test"));
        assert_eq!(Message::new(
            Some("test@test"), "PING", vec![], Some("data")
        ).unwrap().get_source_nickname(), Some("test"));
        assert_eq!(Message::new(
            Some("test"), "PING", vec![], Some("data")
        ).unwrap().get_source_nickname(), Some("test"));
    }

    #[test]
    fn into_string() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!(&message.into_string()[..], "PRIVMSG test :Testing!\r\n");
        let message = Message {
            tags: None,
            prefix: Some(format!("test!test@test")),
            command: PRIVMSG(format!("test"), format!("Still testing!")),
        };
        assert_eq!(&message.into_string()[..], ":test!test@test PRIVMSG test :Still testing!\r\n");
    }

    #[test]
    fn from_string() {
        let message = Message {
            tags: None,
            prefix: None,
            command: PRIVMSG(format!("test"), format!("Testing!")),
        };
        assert_eq!("PRIVMSG test :Testing!\r\n".parse(), Ok(message));
        let message = Message {
            tags: None,
            prefix: Some(format!("test!test@test")),
            command: PRIVMSG(format!("test"), format!("Still testing!")),
        };
        assert_eq!(":test!test@test PRIVMSG test :Still testing!\r\n".parse(), Ok(message));
        let message = Message {
            tags: Some(vec![Tag(format!("aaa"), Some(format!("bbb"))),
                            Tag(format!("ccc"), None),
                            Tag(format!("example.com/ddd"), Some(format!("eee")))]),
            prefix: Some(format!("test!test@test")),
            command: PRIVMSG(format!("test"), format!("Testing with tags!")),
        };
        assert_eq!("@aaa=bbb;ccc;example.com/ddd=eee :test!test@test PRIVMSG test :Testing with \
                    tags!\r\n".parse(), Ok(message))
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
                format!("COMMAND"), vec![format!("ARG:test")], Some(format!("Testing!"))
            ),
        };
        let msg: Message = ":test!test@test COMMAND ARG:test :Testing!\r\n".into();
        assert_eq!(msg, message);
    }

    #[test]
    #[should_panic]
    fn to_message_invalid_format() {
        let _: Message = ":invalid :message".into();
    }
}
