//! A module providing an enum for a message prefix.
use std::borrow::ToOwned;
use std::string;
use std::str::FromStr;
use std::fmt;

/// The Prefix indicates "the true origin of the message", according to the server.
///
/// Warning: Avoid constructing a `Nickname(nickname, None, Some(hostname))`, but
/// `Nickname(nickname, Some("".to_owned()), Some(hostname))` works reliably.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Prefix {
    /// servername
    ServerName(String),
    /// nickname [ ["!" username] "@" hostname ]
    Nickname(String, String, String),
}

impl Prefix {
    /// Creates a prefix by parsing a string.
    ///
    /// # Example
    /// ```
    /// # extern crate irc;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// Prefix::new_from_str("nickname!username@hostname");
    /// Prefix::new_from_str("example.com");
    /// # }
    /// ```
    pub fn new_from_str(s: &str) -> Prefix {
        #[derive(Copy, Clone, Eq, PartialEq)]
        enum Active {
            Name = 0,
            User = 1,
            Host = 2,
        }

        let mut name = String::new();
        let mut user = String::new();
        let mut host = String::new();
        let mut active = Active::Name;

        for c in s.chars() {
            match c {
                // We consider the '.' to be a ServerName except if a ! has already
                // been encountered.
                '.' if active == Active::Name => return Prefix::ServerName(s.to_owned()),

                '!' if active == Active::Name => {
                    active = Active::User;
                },

                // The '@' is not special until we've started the username
                // portion
                '@' if active == Active::User => {
                    active = Active::Host;
                },

                _ => {
                    // Push onto the latest buffer
                    match active {
                        Active::Name => &mut name,
                        Active::User => &mut user,
                        Active::Host => &mut host,
                    }.push(c)
                }
            }
        }

        Prefix::Nickname(name, user, host)
    }
}

/// This implementation never returns an error and is isomorphic with `Display`.
impl FromStr for Prefix {
    type Err = string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Prefix::new_from_str(s))
    }
}

/// This is isomorphic with `FromStr`
impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Prefix::ServerName(name) => write!(f, "{}", name),
            Prefix::Nickname(name, user, host) => match (&name[..], &user[..], &host[..]) {
                ("", "", "") => write!(f, ""),
                (name, "", "") => write!(f, "{}", name),
                (name, user, "") => write!(f, "{}!{}", name, user),
                // This case shouldn't happen normally, but user!@host is invalid, so we
                // format it without the host
                (name, "", _host) => write!(f, "{}", name),
                (name, user, host) => write!(f, "{}!{}@{}", name, user, host),
            },
        }
    }
}

impl<'a> From<&'a str> for Prefix {
    fn from(s: &str) -> Self {
        Prefix::new_from_str(s)
    }
}

#[cfg(test)]
mod test {
    use super::Prefix::{self, ServerName, Nickname};

    // Checks that str -> parsed -> Display doesn't lose data
    fn test_parse(s: &str) -> Prefix {
        let prefix = Prefix::new_from_str(s);
        let s2 = format!("{}", prefix);
        assert_eq!(s, &s2);
        prefix
    }

    #[test]
    fn print() {
        let s = format!("{}", Nickname("nick".into(), "".into(), "".into()));
        assert_eq!(&s, "nick");
        let s = format!("{}", Nickname("nick".into(), "user".into(), "".into()));
        assert_eq!(&s, "nick!user");
        let s = format!("{}", Nickname("nick".into(), "user".into(), "host".into()));
        assert_eq!(&s, "nick!user@host");
    }

    #[test]
    fn parse_word() {
        assert_eq!(
            test_parse("only_nick"),
            Nickname("only_nick".into(), String::new(), String::new())
        )
    }

    #[test]
    fn parse_host() {
        assert_eq!(
            test_parse("host.tld"),
            ServerName("host.tld".into())
        )
    }

    #[test]
    fn parse_nick_user() {
        assert_eq!(
            test_parse("test!nick"),
            Nickname("test".into(), "nick".into(), String::new())
        )
    }

    #[test]
    fn parse_nick_user_host() {
        assert_eq!(
            test_parse("test!nick@host"),
            Nickname("test".into(), "nick".into(), "host".into())
        )
    }

    #[test]
    fn parse_dot_and_symbols() {
        assert_eq!(
            test_parse("test.net@something"),
            ServerName("test.net@something".into())
        )
    }

    #[test]
    fn parse_danger_cases() {
        assert_eq!(
            test_parse("name@name!user"),
            Nickname("name@name".into(), "user".into(), String::new())
        );
        assert_eq!(
            test_parse("name!@"),
            Nickname("name".into(), "".into(), "".into())
        );
        assert_eq!(
            test_parse("name!@hostname"),
            Nickname("name".into(), "".into(), "hostname".into())
        );
        assert_eq!(
            test_parse("name!.user"),
            Nickname("name".into(), ".user".into(), String::new())
        );
        assert_eq!(
            test_parse("name!user.user"),
            Nickname("name".into(), "user.user".into(), String::new())
        );
        assert_eq!(
            test_parse("name!user@host.host"),
            Nickname("name".into(), "user".into(), "host.host".into())
        );
        assert_eq!(
            test_parse("!user"),
            Nickname("".into(), "user".into(), String::new())
        );
        assert_eq!(
            test_parse("!@host.host"),
            Nickname("".into(), "".into(), "host.host".into())
        );
    }
}
