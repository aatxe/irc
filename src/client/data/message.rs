//! Messages to and from the server.
#![unstable = "New features were added recently."]
use std::borrow::ToOwned;
use std::str::FromStr;

/// IRC Message data.
#[stable]
#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    /// The message prefix (or source) as defined by [RFC 2812](http://tools.ietf.org/html/rfc2812).
    #[stable]
    pub prefix: Option<String>,
    /// The IRC command as defined by [RFC 2812](http://tools.ietf.org/html/rfc2812).
    #[stable]
    pub command: String,
    /// The command arguments.
    #[stable]
    pub args: Vec<String>,
    /// The message suffix as defined by [RFC 2812](http://tools.ietf.org/html/rfc2812).
    /// This is the only part of the message that is allowed to contain spaces.
    #[stable]
    pub suffix: Option<String>,
}

#[stable]
impl Message {
    /// Creates a new Message.
    #[stable]
    pub fn new(prefix: Option<&str>, command: &str, args: Option<Vec<&str>>, suffix: Option<&str>)
        -> Message {
        Message {
            prefix: prefix.map(|s| s.to_owned()),
            command: command.to_owned(),
            args: args.map_or(Vec::new(), |v| v.iter().map(|&s| s.to_owned()).collect()),
            suffix: suffix.map(|s| s.to_owned()),
        }
    }

    /// Gets the nickname of the message source, if it exists. 
    #[stable]
    pub fn get_source_nickname(&self) -> Option<&str> {
        self.prefix.as_ref().and_then(|s| s.find('!').map(|i| &s[..i]))
    }

    /// Converts a Message into a String according to the IRC protocol.
    #[stable]
    pub fn into_string(&self) -> String {
        let mut ret = String::new();
        if let Some(ref prefix) = self.prefix {
            ret.push(':');
            ret.push_str(&prefix[]);
            ret.push(' ');
        }
        ret.push_str(&self.command[]);
        for arg in self.args.iter() {
            ret.push(' ');
            ret.push_str(&arg[]);
        }
        if let Some(ref suffix) = self.suffix {
            ret.push_str(" :");
            ret.push_str(&suffix[]);
        }
        ret.push_str("\r\n");
        ret
    }
}

impl ToMessage for Message {
    fn to_message(&self) -> Message {
        self.clone()
    }
}

impl FromStr for Message {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Message, &'static str> {
        let mut state = s.clone();
        if s.len() == 0 { return Err("Cannot parse an empty string as a message.") }
        let prefix = if state.starts_with(":") {
            let prefix = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i+1..]);
            prefix
        } else {
            None
        };
        let suffix = if state.contains(":") {
            let suffix = state.find(':').map(|i| &state[i+1..state.len()-2]);
            state = state.find(':').map_or("", |i| &state[..i]);
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
        Ok(Message::new(prefix, command, if args.len() > 0 { Some(args) } else { None }, suffix))
    }
}

/// A trait representing the ability to be converted into a Message.
#[stable]
pub trait ToMessage {
    /// Converts this to a Message.
    fn to_message(&self) -> Message;
}

impl<'a> ToMessage for &'a str {
    fn to_message(&self) -> Message {
        self.parse().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::{Message, ToMessage};

    #[test]
    fn new() {
        let message = Message {
            prefix: None,
            command: format!("PRIVMSG"),
            args: vec![format!("test")],
            suffix: Some(format!("Testing!")),
        };
        assert_eq!(Message::new(None, "PRIVMSG", Some(vec!["test"]), Some("Testing!")), message);
    }

    #[test]
    fn get_source_nickname() {
        assert_eq!(Message::new(None, "PING", None, None).get_source_nickname(), None);
        assert_eq!(Message::new(
            Some("irc.test.net"), "PING", None, None
        ).get_source_nickname(), None);
        assert_eq!(Message::new(
            Some("test!test@test"), "PING", None, None
        ).get_source_nickname(), Some("test"));
    }

    #[test]
    fn into_string() {
        let message = Message {
            prefix: None,
            command: format!("PRIVMSG"),
            args: vec![format!("test")],
            suffix: Some(format!("Testing!")),
        };
        assert_eq!(&message.into_string()[], "PRIVMSG test :Testing!\r\n");
        let message = Message {
            prefix: Some(format!("test!test@test")),
            command: format!("PRIVMSG"),
            args: vec![format!("test")],
            suffix: Some(format!("Still testing!")),
        };
        assert_eq!(&message.into_string()[], ":test!test@test PRIVMSG test :Still testing!\r\n");
    }

    #[test]
    fn from_string() {
        let message = Message {
            prefix: None,
            command: format!("PRIVMSG"),
            args: vec![format!("test")],
            suffix: Some(format!("Testing!")),
        };
        assert_eq!("PRIVMSG test :Testing!\r\n".parse(), Ok(message));
        let message = Message {
            prefix: Some(format!("test!test@test")),
            command: format!("PRIVMSG"),
            args: vec![format!("test")],
            suffix: Some(format!("Still testing!")),
        };
        assert_eq!(":test!test@test PRIVMSG test :Still testing!\r\n".parse(), Ok(message));
    }

    #[test]
    fn to_message() {
        let message = Message {
            prefix: None,
            command: format!("PRIVMSG"),
            args: vec![format!("test")],
            suffix: Some(format!("Testing!")),
        };
        assert_eq!("PRIVMSG test :Testing!\r\n".to_message(), message);
        let message = Message {
            prefix: Some(format!("test!test@test")),
            command: format!("PRIVMSG"),
            args: vec![format!("test")],
            suffix: Some(format!("Still testing!")),
        };
        assert_eq!(":test!test@test PRIVMSG test :Still testing!\r\n".to_message(), message);
    }

    #[test]
    #[should_fail]
    fn to_message_invalid_format() {
        ":invalid :message".to_message();
    }
}
